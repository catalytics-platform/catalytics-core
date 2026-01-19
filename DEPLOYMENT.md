# Catalytics Core - EKS Deployment Guide

This guide covers deploying the Catalytics Core Rust application to your Fiji Solutions EKS infrastructure.

## 🏗️ Infrastructure Overview

**Environments:**
- **Staging**: `fiji-staging-cluster` → `staging.api.app.catalytics.pro`
- **Production**: `fiji-prod-cluster` → `api.app.catalytics.pro`

**Deployment Features:**
- ARM64-optimized Docker containers for Graviton2 instances
- Database migrations via init containers
- Auto-scaling with HPA
- SSL termination with ACM certificates
- Health checks and readiness probes

## 🚀 Quick Start

### 1. Prerequisites

Ensure you have the required GitHub secrets configured:

**Staging Secrets:**
- `STAGING_DATABASE_URL` - PostgreSQL connection string
- `STAGING_CERTIFICATE_ARN` - ACM certificate ARN for staging domain
- `STAGING_JUPITER_API_KEY` - Jupiter API key for staging
- `STAGING_CATALYTICS_API_BASE_URL` - Staging Catalytics API URL
- `STAGING_CATALYTICS_CORE_API_BASE_URL` - Staging Core API URL
- `STAGING_MAILCHIMP_API_KEY` - Mailchimp API key for staging
- `STAGING_MAILCHIMP_LIST_ID` - Mailchimp list ID for staging
- `STAGING_MAILCHIMP_SERVER_PREFIX` - Mailchimp server prefix for staging

**Production Secrets:**
- `PROD_DATABASE_URL` - PostgreSQL connection string  
- `PROD_CERTIFICATE_ARN` - ACM certificate ARN for production domain
- `PROD_JUPITER_API_KEY` - Jupiter API key for production
- `PROD_CATALYTICS_API_BASE_URL` - Production Catalytics API URL
- `PROD_CATALYTICS_CORE_API_BASE_URL` - Production Core API URL
- `PROD_MAILCHIMP_API_KEY` - Mailchimp API key for production
- `PROD_MAILCHIMP_LIST_ID` - Mailchimp list ID for production
- `PROD_MAILCHIMP_SERVER_PREFIX` - Mailchimp server prefix for production

**Shared Secrets:**
- `AWS_ACCESS_KEY_ID_PROD` - AWS access key
- `AWS_SECRET_ACCESS_KEY_PROD` - AWS secret key
- `JUPITER_API_BASE_URL` - Jupiter API base URL (https://api.jup.ag)
- `CATICS_TOKEN_ADDRESS` - Catics token contract address
- `JUP_TOKEN_ADDRESS` - JUP token contract address

### 2. Automatic Deployment

**Deploy to Staging:**
```bash
# Push to main branch triggers staging deployment
git push origin main
```

**Deploy to Production:**
```bash
# Manual workflow dispatch required for production
# Go to GitHub Actions → Deploy Catalytics Core → Run workflow → Select "production"
```

### 3. Manual Deployment

**Build and deploy locally:**
```bash
# Build the application image
docker buildx build --platform linux/arm64 -t catalytics-core:local .

# Build the migration image  
docker buildx build --platform linux/arm64 -f Dockerfile.migrations -t catalytics-migrations:local .

# Configure kubectl
aws eks update-kubeconfig --region eu-central-1 --name fiji-staging-cluster

# Deploy to staging
kubectl create namespace catalytics-core-staging --dry-run=client -o yaml | kubectl apply -f -

# Set environment variables and deploy
export NAMESPACE=catalytics-core-staging
export ENVIRONMENT=staging
export IMAGE=your-ecr-url/fiji-cors-anywhere:catalytics-core-latest
export MIGRATION_IMAGE=your-ecr-url/fiji-cors-anywhere:catalytics-migrations-latest
export DATABASE_URL="postgres://user:pass@host:5432/db"
export CERTIFICATE_ARN="arn:aws:acm:eu-central-1:account:certificate/cert-id"
export DOMAIN=staging.api.app.catalytics.pro
export REPLICAS=1
export MIN_REPLICAS=1
export MAX_REPLICAS=3
export ALB_GROUP_NAME=staging-shared-alb
export RUST_LOG="catalytics_core=info,sqlx=warn"

# Apply manifests
envsubst < k8s/deployment.yaml | kubectl apply -f -
envsubst < k8s/service.yaml | kubectl apply -f -  
envsubst < k8s/ingress.yaml | kubectl apply -f -
```

## 🔧 Application Configuration

### Environment Variables

| Variable | Description                  | Required |
|----------|------------------------------|----------|
| `DATABASE_URL` | PostgreSQL connection string | Yes |
| `JUPITER_API_BASE_URL` | Jupiter API base URL | Yes |
| `JUPITER_API_KEY` | Jupiter API authentication key | Yes |
| `CATICS_TOKEN_ADDRESS` | Catics token contract address | Yes |
| `JUP_TOKEN_ADDRESS` | JUP token contract address | Yes |
| `CATALYTICS_API_BASE_URL` | Catalytics API base URL | Yes |
| `MAILCHIMP_API_KEY` | Mailchimp API authentication key | Yes |
| `MAILCHIMP_LIST_ID` | Mailchimp audience/list identifier | Yes |
| `MAILCHIMP_SERVER_PREFIX` | Mailchimp server region (us9/us17) | Yes |
| `PORT` | Server port (default: 3000) | No |
| `RUST_LOG` | Logging configuration | No |

### Mailchimp Integration

The application integrates with Mailchimp for email subscription management:

**Field Mapping:**
- `MMERGE1` → Beta Applicant ID (number)
- `MMERGE2` → Truncated Public Key (text)  
- `MMERGE3` → Referral Code (text)
- `MMERGE4` → Referred By ID (optional)

**Sync Behavior:**
- Adding email creates subscribed member
- Changing email validates new address first, then updates
- Removing email deletes member from Mailchimp
- All validation errors return raw Mailchimp messages

### Health Endpoints

- **Health Check**: `GET /api/k8s/health` - Basic health status
- **Readiness Check**: `GET /api/k8s/ready` - Application ready status

## 🗃️ Database Migrations

Database migrations run automatically via init containers before the main application starts.

**Migration Process:**
1. Init container runs `sqlx migrate run`
2. SQLx migrations in `/migrations/` directory are applied
3. Main application starts only after successful migration

**Manual Migration:**
```bash
# Install SQLx CLI (if not already installed)
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations manually
sqlx migrate run
```

**SQLx Offline Mode (for Docker builds):**
```bash
# If you encounter SQLx compilation issues in Docker builds, 
# prepare offline data with a local database first:
cargo sqlx prepare

# This generates sqlx-data.json for offline compilation
# Then uncomment SQLX_OFFLINE=true in Dockerfile
```

## 📁 File Structure

```
catalytics-core/
├── .github/workflows/
│   └── deploy.yml              # CI/CD pipeline
├── k8s/
│   ├── deployment.yaml         # Kubernetes deployment with init container
│   ├── service.yaml           # Service definition
│   └── ingress.yaml           # ALB ingress with SSL
├── migrations/
│   └── *.sql                  # SQLx migration files
├── src/
│   └── ...                    # Application source
├── Dockerfile                 # Main application image
├── Dockerfile.migrations      # Migration runner image
└── .dockerignore             # Docker build exclusions
```

## 🔍 Troubleshooting

### Common Issues

**1. Migration Failures**
```bash
# Check init container logs
kubectl logs -l app=catalytics-core -c db-migrate -n catalytics-core-staging
```

**2. Application Not Starting**
```bash
# Check application logs
kubectl logs -l app=catalytics-core -c catalytics-core -n catalytics-core-staging

# Check events
kubectl get events -n catalytics-core-staging --sort-by='.lastTimestamp'
```

**3. Ingress Issues**
```bash
# Check ALB creation
kubectl describe ingress catalytics-core-ingress -n catalytics-core-staging

# Verify certificate
aws acm describe-certificate --certificate-arn YOUR_CERT_ARN --region eu-central-1
```

**4. Mailchimp Integration Issues**
```bash
# Check for Mailchimp-related errors in logs
kubectl logs -l app=catalytics-core -n catalytics-core-staging | grep -i mailchimp

# Common Mailchimp errors:
# - "merge fields were invalid" → Check list configuration in Mailchimp
# - "looks fake or invalid" → Email validation failed
# - "cannot be removed" → Contact in restricted state (bounced/archived)
```

### Health Check Commands

```bash
# Test health endpoint locally
curl http://localhost:3000/api/k8s/health

# Test via kubectl port-forward
kubectl port-forward service/catalytics-core-service 3000:80 -n catalytics-core-staging
curl http://localhost:3000/api/k8s/health

# Check pod readiness
kubectl get pods -l app=catalytics-core -n catalytics-core-staging
```

## 🏷️ Resource Management

**Staging Configuration:**
- Replicas: 1
- CPU: 100m request, 200m limit
- Memory: 128Mi request, 256Mi limit
- Auto-scaling: 1-3 replicas

**Production Configuration:**
- Replicas: 2  
- CPU: 100m request, 200m limit
- Memory: 128Mi request, 256Mi limit
- Auto-scaling: 2-10 replicas

## 🔐 Security

- Non-root container execution (user ID 1000)
- Read-only root filesystem capability
- No privilege escalation
- Kubernetes secrets for sensitive data
- SSL/TLS termination at ALB level

---

For infrastructure questions, refer to the main infrastructure repository documentation.
