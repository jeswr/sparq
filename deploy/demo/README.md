<!-- [SONNET-4.6] sq-80afk — ephemeral LWS + CSS Cloud Run demo. -->

# Ephemeral Solid/LWS demo on Cloud Run

This is a public, throwaway test environment, not a production deployment. It runs two
independent, in-memory Cloud Run services that scale to zero: the experimental sparq LWS
server and a Community Solid Server (CSS) identity provider.

> **Demo banner:** Everything in `/playground/` is public-readable. Authenticated visitors
> share it and are not isolated from one another. Accounts, keys, replay state, and data are
> temporary and may vanish after idle scale-down. If anything returns 401 or disappears,
> re-register; that is the demo working as designed. Use only throwaway identities and data.

## Visitor flow

1. Open the CSS service URL and register a throwaway account. Email is not verified.
2. Use a Solid client to sign in through CSS and obtain a DPoP-bound token.
3. Read or write the shared `https://<sparq-lws-demo-url>/playground/` container.

Anonymous writes remain denied. Visitors cannot take control of the playground ACL, but all
authenticated visitors can read and modify one another's playground data.

## Wipe semantics

Cloud Run reclaims idle instances on a heuristic schedule; there is no guaranteed wipe
deadline. The LWS and CSS services scale down independently. If CSS is reclaimed, old tokens
and hosted WebIDs stop working. If LWS is reclaimed, the empty playground is seeded again.
The in-memory DPoP replay store is also reset, so this posture is suitable only for a
throwaway demo.

## Deploy

Set the project and region, render the deterministic Cloud Run URLs, then replace both
services:

```bash
PROJECT_ID="$(gcloud config get-value project)"
PROJECT_NUMBER="$(gcloud projects describe "${PROJECT_ID}" --format='value(projectNumber)')"
REGION="us-central1"
SPARQ_LWS_DIGEST="<64-character sha256 digest>"
CSS_DIGEST="<64-character sha256 digest>"
gcloud services enable run.googleapis.com --project="${PROJECT_ID}"
gcloud iam service-accounts create sparq-demo-sa \
  --display-name="SPARQ public demo runtime" --project="${PROJECT_ID}"

render_dir="$(mktemp -d)"
trap 'rm -rf "${render_dir}"' EXIT
sed -e "s/PROJECT_ID/${PROJECT_ID}/g" -e "s/PROJECT_NUMBER/${PROJECT_NUMBER}/g" \
  -e "s/REGION/${REGION}/g" -e "s/CSS_DIGEST/${CSS_DIGEST}/g" \
  deploy/demo/css-idp.yaml >"${render_dir}/css-idp.yaml"
sed -e "s/PROJECT_ID/${PROJECT_ID}/g" -e "s/PROJECT_NUMBER/${PROJECT_NUMBER}/g" \
  -e "s/REGION/${REGION}/g" -e "s/SPARQ_LWS_DIGEST/${SPARQ_LWS_DIGEST}/g" \
  deploy/demo/sparq-lws-demo.yaml >"${render_dir}/sparq-lws-demo.yaml"

bash deploy/demo/check.sh \
  "${render_dir}/sparq-lws-demo.yaml" "${render_dir}/css-idp.yaml"

gcloud run services replace "${render_dir}/css-idp.yaml" \
  --region="${REGION}" --project="${PROJECT_ID}"
gcloud run services replace "${render_dir}/sparq-lws-demo.yaml" \
  --region="${REGION}" --project="${PROJECT_ID}"
```

Resolve both digest variables before deployment (for example with `docker buildx imagetools
inspect`). The render step replaces the committed placeholders with immutable image digests,
so an upstream tag change cannot silently alter the demo.

Both services intentionally use request-based billing, `minScale: 0`, `maxScale: 1`, and a
TCP startup probe. Do not copy the warm-instance and custom-health-probe posture from
`deploy/gcp/`; that template serves a different purpose.

Create a Google Cloud budget and alert for the demo project before making the services
public. Scaling limits cap compute concurrency but do not cap spend, and budget alerts
notify rather than automatically disabling billing.

The dedicated runtime service account intentionally has no project roles because neither
demo service needs access to Google Cloud APIs.

Run the structural check without arguments to validate the committed templates:

```bash
bash deploy/demo/check.sh
```
