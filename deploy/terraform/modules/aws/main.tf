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
# R5: empty TaskRole + scoped ExecutionRole (logs + own secret only).
# R6: ALB SG allows 443 inbound; task SG allows app port only from ALB SG.
# R7: ALB target-group health check on var.health_path.
# R8: ReadonlyRootFilesystem on task; non-root not overridden.
# R9: open-by-default caveat in header comment above.
#
# DECISION (no EC2 fallback): Fargate managed compute — no OS patching.
# EC2 fallback noted as discovered work per brief.
#
# NOTE: ALB + ACM certificate ARN is required for HTTPS (R3). This module has
# no public plaintext mode.

terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = ">= 5.0.0, < 6.0.0"
    }
  }
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
  vpc_id             = var.vpc_id != "" ? var.vpc_id : data.aws_vpc.default[0].id
  default_subnet_ids = var.vpc_id == "" ? data.aws_subnets.default[0].ids : []
  alb_subnet_ids     = length(var.alb_subnet_ids) > 0 ? var.alb_subnet_ids : local.default_subnet_ids
  task_subnet_ids    = length(var.task_subnet_ids) > 0 ? var.task_subnet_ids : local.alb_subnet_ids
}

# ---------------------------------------------------------------------------
# Secrets Manager — auth token (R1 / R4)
# ---------------------------------------------------------------------------

resource "aws_secretsmanager_secret" "auth_token" {
  count = var.server == "sparq-server" ? 1 : 0

  name                    = "${var.name}-auth-token"
  description             = "sparq-server SPARQ_AUTH_TOKEN — injected via ECS secrets block; never a plaintext env literal (R4)"
  recovery_window_in_days = 7

  tags = {
    ManagedBy = "sparq-terraform"
    Server    = var.server
  }
}

resource "aws_secretsmanager_secret_version" "auth_token" {
  count = var.server == "sparq-server" ? 1 : 0

  secret_id     = aws_secretsmanager_secret.auth_token[0].id
  secret_string = var.auth_token

  lifecycle {
    # [GPT-5.6] Direct child-module callers cannot create an open server with
    # a missing or trivially weak token.
    precondition {
      condition     = try(length(var.auth_token) >= 32, false)
      error_message = "sparq-server requires auth_token with at least 32 characters."
    }
  }
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

# Execution role: write its log stream and read its own secret only. Public
# GHCR pulls do not need the wildcard-heavy ECR managed execution policy.
resource "aws_iam_role" "execution" {
  name               = "${var.name}-exec-role"
  assume_role_policy = data.aws_iam_policy_document.ecs_assume.json
  tags = {
    ManagedBy = "sparq-terraform"
  }
}

data "aws_iam_policy_document" "execution" {
  statement {
    actions = [
      "logs:CreateLogStream",
      "logs:PutLogEvents",
    ]
    resources = ["${aws_cloudwatch_log_group.sparq.arn}:log-stream:*"]
  }

  # [GPT-5.6] LWS has no bearer-token secret, so its execution identity gets
  # no Secrets Manager permission at all.
  dynamic "statement" {
    for_each = var.server == "sparq-server" ? [1] : []
    content {
      actions   = ["secretsmanager:GetSecretValue"]
      resources = [aws_secretsmanager_secret.auth_token[0].arn]
    }
  }
}

resource "aws_iam_role_policy" "execution" {
  name   = "${var.name}-exec"
  role   = aws_iam_role.execution.id
  policy = data.aws_iam_policy_document.execution.json
}

# Dedicated task role intentionally has no permissions; the ECS agent writes
# logs and resolves secrets through the separate execution role.
resource "aws_iam_role" "task" {
  name               = "${var.name}-task-role"
  assume_role_policy = data.aws_iam_policy_document.ecs_assume.json
  tags = {
    ManagedBy = "sparq-terraform"
  }
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

# [GPT-5.6] Security groups are rule-free shells; standalone rule resources
# make every source/destination explicit and avoid circular inline references.
resource "aws_security_group" "alb" {
  name        = "${var.name}-alb-sg"
  description = "sparq ALB: accepts HTTPS from internet; egress to task SG on app port only"
  vpc_id      = local.vpc_id

  tags = {
    ManagedBy = "sparq-terraform"
  }
}

# Task SG: accept app port only from ALB SG (source-based, not CIDR, R6)
resource "aws_security_group" "task" {
  name        = "${var.name}-task-sg"
  description = "sparq task: accept app port from ALB SG only; egress internet"
  vpc_id      = local.vpc_id

  tags = {
    ManagedBy = "sparq-terraform"
  }
}

resource "aws_vpc_security_group_ingress_rule" "alb_https_ipv4" {
  security_group_id = aws_security_group.alb.id
  description       = "Public HTTPS"
  cidr_ipv4         = "0.0.0.0/0"
  from_port         = 443
  to_port           = 443
  ip_protocol       = "tcp"
}

resource "aws_vpc_security_group_ingress_rule" "alb_redirect_ipv4" {
  security_group_id = aws_security_group.alb.id
  description       = "Public HTTP redirect only"
  cidr_ipv4         = "0.0.0.0/0"
  from_port         = 80
  to_port           = 80
  ip_protocol       = "tcp"
}

resource "aws_vpc_security_group_egress_rule" "alb_to_task" {
  security_group_id            = aws_security_group.alb.id
  description                  = "App traffic to ECS tasks only"
  referenced_security_group_id = aws_security_group.task.id
  from_port                    = var.container_port
  to_port                      = var.container_port
  ip_protocol                  = "tcp"
}

resource "aws_vpc_security_group_ingress_rule" "task_from_alb" {
  security_group_id            = aws_security_group.task.id
  description                  = "App traffic from ALB only"
  referenced_security_group_id = aws_security_group.alb.id
  from_port                    = var.container_port
  to_port                      = var.container_port
  ip_protocol                  = "tcp"
}

# HTTPS is sufficient for GHCR pulls, CloudWatch Logs, Secrets Manager, and
# the LWS production-only HTTPS issuer/WebID fetches. No all-protocol egress.
resource "aws_vpc_security_group_egress_rule" "task_https" {
  security_group_id = aws_security_group.task.id
  description       = "HTTPS dependencies only"
  cidr_ipv4         = "0.0.0.0/0"
  from_port         = 443
  to_port           = 443
  ip_protocol       = "tcp"
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
  # sparq-server specific env (SPARQ_AUTH_TOKEN injected via secrets block)
  sparq_server_env = [
    {
      name  = "SPARQ_ALLOW_REMOTE"
      value = "1"
    },
    {
      # [GPT-5.6] Gate reads as well as writes; /health remains ungated.
      name  = "SPARQ_AUTH_TOKEN_READ"
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
      valueFrom = aws_secretsmanager_secret.auth_token[0].arn
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
    merge({
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

      # [GPT-5.6] Neither server needs Linux capabilities on its high port.
      linuxParameters = {
        capabilities = {
          drop = ["ALL"]
        }
        initProcessEnabled = true
      }

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

      }, var.server == "sparq-server" ? {
      # [GPT-5.6] Exec form is required because the image is distroless and has
      # no shell. LWS relies on the ALB /readyz check because its image has no
      # self-probe command.
      healthCheck = {
        command = [
          "CMD",
          "/usr/local/bin/sparq-server",
          "--health-probe"
        ]
        interval    = 30
        timeout     = 5
        retries     = 3
        startPeriod = 60
      }
    } : {})
  ])

  dynamic "volume" {
    for_each = var.server == "sparq-server" ? [1] : []
    content {
      name = "data"
    }
  }

  tags = {
    ManagedBy = "sparq-terraform"
    Server    = var.server
  }

  lifecycle {
    precondition {
      condition = var.server != "lws" || (
        startswith(var.solid_server_base_url, "https://") &&
        startswith(var.solid_server_trusted_issuer, "https://")
      )
      error_message = "lws requires HTTPS solid_server_base_url and solid_server_trusted_issuer values."
    }
    precondition {
      condition = (
        var.server == "sparq-server" && var.container_port == 3030
        ) || (
        var.server == "lws" && var.container_port == 3000
      )
      error_message = "container_port must be 3030 for sparq-server or 3000 for lws."
    }
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
  subnets            = local.alb_subnet_ids

  # Enable deletion protection in production deployments
  enable_deletion_protection = var.enable_deletion_protection

  tags = {
    ManagedBy = "sparq-terraform"
  }

  lifecycle {
    precondition {
      condition     = length(distinct(local.alb_subnet_ids)) >= 2
      error_message = "An internet-facing ALB requires at least two subnet IDs in distinct Availability Zones."
    }
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
  load_balancer_arn = aws_lb.sparq.arn
  port              = 443
  protocol          = "HTTPS"
  ssl_policy        = "ELBSecurityPolicy-TLS13-1-2-2021-06"
  certificate_arn   = var.acm_certificate_arn

  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.sparq.arn
  }

  lifecycle {
    # [GPT-5.6] ALB can use only an ACM certificate from its own account and
    # region; catch cross-account/region ARNs during planning.
    precondition {
      condition     = try(split(":", var.acm_certificate_arn)[3] == data.aws_region.current.name, false)
      error_message = "acm_certificate_arn must be in the AWS provider region."
    }
    precondition {
      condition     = try(split(":", var.acm_certificate_arn)[4] == data.aws_caller_identity.current.account_id, false)
      error_message = "acm_certificate_arn must belong to the active AWS account."
    }
  }
}

# HTTP → HTTPS redirect (R3); no plaintext content served
resource "aws_lb_listener" "http_redirect" {
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

# [GPT-5.6] ACM cannot issue a certificate for the generated ALB hostname.
# Create the application DNS record when Route 53 owns the zone; otherwise the
# caller must create the equivalent alias/CNAME with its external DNS provider.
resource "aws_route53_record" "public" {
  count = var.route53_zone_id != "" ? 1 : 0

  zone_id = var.route53_zone_id
  name    = var.public_hostname
  type    = "A"

  alias {
    name                   = aws_lb.sparq.dns_name
    zone_id                = aws_lb.sparq.zone_id
    evaluate_target_health = true
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

  health_check_grace_period_seconds = 60

  network_configuration {
    subnets         = local.task_subnet_ids
    security_groups = [aws_security_group.task.id]
    # [GPT-5.6] Default-VPC tasks need a public IP to pull GHCR and reach cloud
    # APIs. Operators with private subnets plus NAT/endpoints can disable it;
    # the task SG never permits public ingress in either mode.
    assign_public_ip = var.assign_public_ip
  }

  load_balancer {
    target_group_arn = aws_lb_target_group.sparq.arn
    container_name   = var.server
    container_port   = var.container_port
  }

  depends_on = [
    aws_lb_listener.https,
    aws_lb_listener.http_redirect,
    aws_iam_role_policy.execution,
  ]

  tags = {
    ManagedBy = "sparq-terraform"
    Server    = var.server
  }

  lifecycle {
    precondition {
      condition     = var.server != "lws" || var.desired_count == 1
      error_message = "lws must use desired_count=1 unless shared Redis replay protection is wired."
    }
  }
}
