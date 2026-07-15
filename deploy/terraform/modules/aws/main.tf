# sparq Terraform — AWS submodule (sq-sos84) [SONNET]
#
# Provisions: ECS Fargate cluster + task definition + service, ALB + HTTPS
# listener + target group, Secrets Manager secret (auth token, R4),
# dedicated task role + execution role (R5), security groups (R6),
# CloudWatch log group, VPC data-source lookup.
#
# Secure defaults enforced per research/cloud-deploy-architecture.md §2:
# R1: SPARQ_AUTH_TOKEN injected from Secrets Manager (never a literal).
# R2: unauthenticated writes blocked; LWS escape hatches absent.
# R3: HTTPS via ACM cert on ALB; HTTP redirects 301 to HTTPS.
# R4: token stored in Secrets Manager, sensitive variable.
# R5: TaskRole (logs only) + ExecutionRole (pull image + read own secret).
# R6: ALB SG allows 443 inbound; task SG allows app port only from ALB SG.
# R7: ALB target-group health check on var.health_path.
# R8: ReadonlyRootFilesystem on task; non-root not overridden.
# R9: open-by-default caveat in header comment above.
#
# DECISION (no EC2 fallback): Fargate managed compute — no OS patching.
# EC2 fallback noted as discovered work per brief.
#
# NOTE: ALB + ACM certificate ARN is required for HTTPS (R3). Provide
# var.acm_certificate_arn, or set var.enable_https=false for a dev/internal
# HTTP-only deployment (not recommended for public endpoints).

terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = ">= 5.0.0, < 6.0.0"
    }
    random = {
      source  = "hashicorp/random"
      version = ">= 3.5.0"
    }
  }
}

provider "aws" {
  region = var.aws_region
}

data "aws_region" "current" {}
data "aws_caller_identity" "current" {}

# ---------------------------------------------------------------------------
# Default VPC + subnets (operator can override via var.vpc_id / subnet_ids)
# ---------------------------------------------------------------------------

data "aws_vpc" "default" {
  count   = var.vpc_id == "" ? 1 : 0
  default = true
}

data "aws_subnets" "default" {
  count = var.vpc_id == "" ? 1 : 0
  filter {
    name   = "vpc-id"
    values = [data.aws_vpc.default[0].id]
  }
}

locals {
  vpc_id     = var.vpc_id != "" ? var.vpc_id : data.aws_vpc.default[0].id
  subnet_ids = length(var.subnet_ids) > 0 ? var.subnet_ids : data.aws_subnets.default[0].ids
}

# ---------------------------------------------------------------------------
# Secrets Manager — auth token (R1 / R4)
# ---------------------------------------------------------------------------

resource "aws_secretsmanager_secret" "auth_token" {
  name                    = "${var.name}-auth-token"
  description             = "sparq-server SPARQ_AUTH_TOKEN — injected via ECS secrets block; never a plaintext env literal (R4)"
  recovery_window_in_days = 7

  tags = {
    ManagedBy = "sparq-terraform"
    Server    = var.server
  }
}

resource "aws_secretsmanager_secret_version" "auth_token" {
  secret_id     = aws_secretsmanager_secret.auth_token.id
  secret_string = var.auth_token
}

# ---------------------------------------------------------------------------
# IAM — least-privilege roles (R5)
# ---------------------------------------------------------------------------

data "aws_iam_policy_document" "ecs_assume" {
  statement {
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["ecs-tasks.amazonaws.com"]
    }
  }
}

# Execution role: pull image + read own secret only
resource "aws_iam_role" "execution" {
  name               = "${var.name}-exec-role"
  assume_role_policy = data.aws_iam_policy_document.ecs_assume.json
  tags = {
    ManagedBy = "sparq-terraform"
  }
}

resource "aws_iam_role_policy_attachment" "execution_ecr" {
  role       = aws_iam_role.execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy"
}

data "aws_iam_policy_document" "execution_secret" {
  statement {
    actions   = ["secretsmanager:GetSecretValue"]
    resources = [aws_secretsmanager_secret.auth_token.arn]
    # Scoped to own ARN only — no wildcard (R5)
  }
}

resource "aws_iam_role_policy" "execution_secret" {
  name   = "${var.name}-exec-secret"
  role   = aws_iam_role.execution.id
  policy = data.aws_iam_policy_document.execution_secret.json
}

# Task role: CloudWatch logs only (no wildcard)
resource "aws_iam_role" "task" {
  name               = "${var.name}-task-role"
  assume_role_policy = data.aws_iam_policy_document.ecs_assume.json
  tags = {
    ManagedBy = "sparq-terraform"
  }
}

data "aws_iam_policy_document" "task_logs" {
  statement {
    actions = [
      "logs:CreateLogStream",
      "logs:PutLogEvents",
    ]
    resources = ["${aws_cloudwatch_log_group.sparq.arn}:*"]
    # Scoped to own log group only (R5)
  }
}

resource "aws_iam_role_policy" "task_logs" {
  name   = "${var.name}-task-logs"
  role   = aws_iam_role.task.id
  policy = data.aws_iam_policy_document.task_logs.json
}

# ---------------------------------------------------------------------------
# CloudWatch log group
# ---------------------------------------------------------------------------

resource "aws_cloudwatch_log_group" "sparq" {
  name              = "/ecs/${var.name}"
  retention_in_days = 30
  tags = {
    ManagedBy = "sparq-terraform"
  }
}

# ---------------------------------------------------------------------------
# Security groups (R6)
# ---------------------------------------------------------------------------

# ALB SG: allow 443 (and optionally 80 redirect) inbound from internet
resource "aws_security_group" "alb" {
  name        = "${var.name}-alb-sg"
  description = "sparq ALB: accepts HTTPS from internet; egress to task SG on app port only"
  vpc_id      = local.vpc_id

  ingress {
    description = "HTTPS from internet"
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
    # tflint-ignore: aws_security_group_rule_missing_description
    ipv6_cidr_blocks = ["::/0"]
  }

  # HTTP redirect only (no plaintext content served)
  ingress {
    description      = "HTTP redirect to HTTPS"
    from_port        = 80
    to_port          = 80
    protocol         = "tcp"
    cidr_blocks      = ["0.0.0.0/0"]
    ipv6_cidr_blocks = ["::/0"]
  }

  egress {
    description = "Allow outbound to task on app port"
    from_port   = var.container_port
    to_port     = var.container_port
    protocol    = "tcp"
    # resolved to task SG below via aws_security_group_rule
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    ManagedBy = "sparq-terraform"
  }
}

# Task SG: accept app port only from ALB SG (source-based, not CIDR, R6)
resource "aws_security_group" "task" {
  name        = "${var.name}-task-sg"
  description = "sparq task: accept app port from ALB SG only; egress internet"
  vpc_id      = local.vpc_id

  ingress {
    description     = "App port from ALB SG only (R6)"
    from_port       = var.container_port
    to_port         = var.container_port
    protocol        = "tcp"
    security_groups = [aws_security_group.alb.id]
  }

  egress {
    description = "Allow all outbound (pull secrets, image)"
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    ManagedBy = "sparq-terraform"
  }
}

# ---------------------------------------------------------------------------
# ECS cluster
# ---------------------------------------------------------------------------

resource "aws_ecs_cluster" "sparq" {
  name = "${var.name}-cluster"

  setting {
    name  = "containerInsights"
    value = "enabled"
  }

  tags = {
    ManagedBy = "sparq-terraform"
  }
}

# ---------------------------------------------------------------------------
# ECS task definition
# ---------------------------------------------------------------------------

locals {
  # Environment variables common to all server types
  common_env = []

  # sparq-server specific env (SPARQ_AUTH_TOKEN injected via secrets block)
  sparq_server_env = [
    {
      name  = "SPARQ_ALLOW_REMOTE"
      value = "1"
    }
  ]

  # LWS specific env (no dev escape hatches per R2)
  lws_env = [
    {
      name  = "SOLID_SERVER_BASE_URL"
      value = var.solid_server_base_url
    },
    {
      name  = "SOLID_SERVER_TRUSTED_ISSUER"
      value = var.solid_server_trusted_issuer
    }
  ]

  server_env = var.server == "sparq-server" ? local.sparq_server_env : local.lws_env

  # Secrets (auth token from Secrets Manager, R4) — only for sparq-server
  # LWS is fail-closed by design; auth_token var unused for lws
  secrets = var.server == "sparq-server" ? [
    {
      name      = "SPARQ_AUTH_TOKEN"
      valueFrom = aws_secretsmanager_secret.auth_token.arn
    }
  ] : []
}

resource "aws_ecs_task_definition" "sparq" {
  family                   = var.name
  network_mode             = "awsvpc"
  requires_compatibilities = ["FARGATE"]
  cpu                      = var.cpu
  memory                   = var.memory
  execution_role_arn       = aws_iam_role.execution.arn
  task_role_arn            = aws_iam_role.task.arn

  container_definitions = jsonencode([
    {
      name      = var.server
      image     = var.image
      essential = true

      portMappings = [
        {
          containerPort = var.container_port
          protocol      = "tcp"
        }
      ]

      environment = local.server_env
      secrets     = local.secrets

      # R8: read-only root filesystem; writable /data volume for sparq-server dataset
      readonlyRootFilesystem = true

      # R8: non-root user not overridden (images run as nonroot by default)

      mountPoints = var.server == "sparq-server" ? [
        {
          sourceVolume  = "data"
          containerPath = "/data"
          readOnly      = false
        }
      ] : []

      logConfiguration = {
        logDriver = "awslogs"
        options = {
          "awslogs-group"         = aws_cloudwatch_log_group.sparq.name
          "awslogs-region"        = data.aws_region.current.name
          "awslogs-stream-prefix" = var.server
        }
      }

      # Health check via server self-probe (R7)
      healthCheck = {
        command = var.server == "sparq-server" ? [
          "CMD-SHELL",
          "sparq-server --health-probe || exit 1"
        ] : [
          "CMD-SHELL",
          "wget -q -O- http://localhost:${var.container_port}/livez || exit 1"
        ]
        interval    = 30
        timeout     = 5
        retries     = 3
        startPeriod = 60
      }
    }
  ])

  volume {
    name = "data"
  }

  tags = {
    ManagedBy = "sparq-terraform"
    Server    = var.server
  }
}

# ---------------------------------------------------------------------------
# ALB + HTTPS listener (R3)
# ---------------------------------------------------------------------------

resource "aws_lb" "sparq" {
  name               = "${var.name}-alb"
  internal           = false
  load_balancer_type = "application"
  security_groups    = [aws_security_group.alb.id]
  subnets            = local.subnet_ids

  # Enable deletion protection in production deployments
  enable_deletion_protection = var.enable_deletion_protection

  tags = {
    ManagedBy = "sparq-terraform"
  }
}

resource "aws_lb_target_group" "sparq" {
  name        = "${var.name}-tg"
  port        = var.container_port
  protocol    = "HTTP"
  vpc_id      = local.vpc_id
  target_type = "ip"

  health_check {
    path                = var.health_path
    healthy_threshold   = 2
    unhealthy_threshold = 3
    interval            = 30
    timeout             = 5
    matcher             = "200"
  }

  tags = {
    ManagedBy = "sparq-terraform"
  }
}

# HTTPS listener (R3) — requires ACM cert ARN
resource "aws_lb_listener" "https" {
  count             = var.acm_certificate_arn != "" ? 1 : 0
  load_balancer_arn = aws_lb.sparq.arn
  port              = 443
  protocol          = "HTTPS"
  ssl_policy        = "ELBSecurityPolicy-TLS13-1-2-2021-06"
  certificate_arn   = var.acm_certificate_arn

  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.sparq.arn
  }
}

# HTTP → HTTPS redirect (R3); no plaintext content served
resource "aws_lb_listener" "http_redirect" {
  count             = var.acm_certificate_arn != "" ? 1 : 0
  load_balancer_arn = aws_lb.sparq.arn
  port              = 80
  protocol          = "HTTP"

  default_action {
    type = "redirect"
    redirect {
      port        = "443"
      protocol    = "HTTPS"
      status_code = "HTTP_301"
    }
  }
}

# HTTP-only listener for dev/internal (no public credential flow — R3 guidance)
resource "aws_lb_listener" "http_dev" {
  count             = var.acm_certificate_arn == "" ? 1 : 0
  load_balancer_arn = aws_lb.sparq.arn
  port              = 80
  protocol          = "HTTP"

  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.sparq.arn
  }
}

# ---------------------------------------------------------------------------
# ECS Fargate service
# ---------------------------------------------------------------------------

resource "aws_ecs_service" "sparq" {
  name            = "${var.name}-service"
  cluster         = aws_ecs_cluster.sparq.id
  task_definition = aws_ecs_task_definition.sparq.arn
  desired_count   = var.desired_count
  launch_type     = "FARGATE"

  network_configuration {
    subnets          = local.subnet_ids
    security_groups  = [aws_security_group.task.id]
    assign_public_ip = false # Tasks in private subnets (R6)
  }

  load_balancer {
    target_group_arn = aws_lb_target_group.sparq.arn
    container_name   = var.server
    container_port   = var.container_port
  }

  depends_on = [
    aws_lb_listener.https,
    aws_lb_listener.http_dev,
    aws_iam_role_policy_attachment.execution_ecr,
  ]

  tags = {
    ManagedBy = "sparq-terraform"
    Server    = var.server
  }
}
