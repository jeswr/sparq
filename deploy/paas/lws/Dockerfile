# [GPT-5.6] Railway config-as-code cannot select an image; consume the canonical image here.
FROM ghcr.io/sparq-org/sparq-lws-core:latest

# The binary itself defaults to loopback, which Railway ingress cannot reach.
ENV SOLID_SERVER_BIND=0.0.0.0:3000
