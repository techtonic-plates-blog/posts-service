# GitHub Actions Workflows for Container Publishing

This directory contains GitHub Actions workflows for building and publishing Docker containers to GitHub Container Registry (GHCR).

## Workflows

### 1. `docker-publish.yml` - Automated Publishing
**Triggers:**
- Push to `main` branch
- Git tags (any tag)
- Manual workflow dispatch

**Description:**
Builds and publishes all four container images to GHCR:
- `ghcr.io/techtonic-plates-blog/posts-service:latest` (main posts service)
- `ghcr.io/techtonic-plates-blog/posts-service-entities:latest` (entities generator)
- `ghcr.io/techtonic-plates-blog/posts-service-migration:latest` (database migrations)
- `ghcr.io/techtonic-plates-blog/posts-service-terraform:latest` (terraform operations)

**Tags:**
- `latest` - for main branch pushes
- `nightly` - for main branch pushes (same as latest)
- `<tag-name>` - for Git tags

### 2. `docker-manual-build.yml` - Manual Builds
**Triggers:**
- Manual workflow dispatch only

**Description:**
Allows selective building of individual containers or all containers with custom options:
- Choose specific container or build all
- Optional custom tag
- Option to build without pushing (for testing)

**Usage:**
1. Go to Actions tab in GitHub
2. Select "Manual Docker Build"
3. Click "Run workflow"
4. Choose container, tag, and push options

### 3. `docker-test-builds.yml` - Pull Request Testing
**Triggers:**
- Pull requests to `main` branch
- Only when relevant files are changed

**Description:**
Tests that all Docker builds complete successfully without pushing to registry. Provides early feedback on build issues.

## Container Images

| Container | Dockerfile | Purpose | Registry Tag Suffix |
|-----------|------------|---------|-------------------|
| posts Service | `container/Containerfile` | Main postsentication service | (none) |
| Entities | `container/Containerfile.entities` | SeaORM entity generator | `-entities` |
| Migration | `container/Containerfile.migration` | Database migration runner | `-migration` |
| Terraform | `container/Containerfile.terraform` | Infrastructure management | `-terraform` |

## Registry postsentication

The workflows use the built-in `GITHUB_TOKEN` with `packages: write` permission to push to GHCR. No additional secrets are required.

## Build Caching

All workflows use GitHub Actions cache to speed up builds:
- `cache-from: type=gha` - Use cache for builds
- `cache-to: type=gha,mode=max` - Save cache after builds

## Usage Examples

### Pulling Images
```bash
# Pull the main posts service
docker pull ghcr.io/techtonic-plates-blog/posts-service:latest

# Pull the migration runner
docker pull ghcr.io/techtonic-plates-blog/posts-service-migration:latest

# Pull specific version
docker pull ghcr.io/techtonic-plates-blog/posts-service:v1.0.0
```

### Using in Docker Compose
```yaml
version: '3.8'
services:
  posts-service:
    image: ghcr.io/techtonic-plates-blog/posts-service:latest
    # ... other config
  
  migration:
    image: ghcr.io/techtonic-plates-blog/posts-service-migration:latest
    # ... other config
```

### Using with Kubernetes
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: posts-service
spec:
  template:
    spec:
      containers:
      - name: posts-service
        image: ghcr.io/techtonic-plates-blog/posts-service:latest
        # ... other config
```

## Troubleshooting

### Build Failures
1. Check the Actions tab for detailed logs
2. Verify Dockerfile syntax
3. Ensure all dependencies are available

### Permission Issues
1. Verify `packages: write` permission is granted
2. Check if repository settings allow package publishing
3. Ensure workflows are run from the main repository (not forks)

### Cache Issues
If builds are slow or failing due to cache:
1. Manually clear cache in repository settings
2. Temporarily disable cache in workflow
3. Check cache size limits
