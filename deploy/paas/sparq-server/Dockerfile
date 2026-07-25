# [GPT-5.6] Railway config-as-code cannot select an image; consume the canonical image here.
FROM ghcr.io/sparq-org/sparq-server:latest

# Override the base image's open bind opt-in. The server permits its public bind only after
# Railway injects SPARQ_AUTH_TOKEN; read-auth makes the entire surface token-gated.
ENV SPARQ_ALLOW_REMOTE=0
ENV SPARQ_AUTH_TOKEN_READ=1
