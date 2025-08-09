# GitHub Repository Setup for Container Publishing

This document outlines the required GitHub repository settings to enable container publishing to GitHub Container Registry (GHCR).

## Required Repository Settings

### 1. Actions Permissions

Navigate to: **Settings → Actions → General**

Ensure the following settings are enabled:
- ✅ **Allow all actions and reusable workflows**
- ✅ **Allow actions created by GitHub**
- ✅ **Allow actions by verified creators**

### 2. Workflow Permissions

Navigate to: **Settings → Actions → General → Workflow permissions**

Set the following:
- ✅ **Read and write permissions** (required for package publishing)
- ✅ **Allow GitHub Actions to create and approve pull requests**

### 3. Package Permissions

Navigate to: **Settings → Code and automation → Packages**

Ensure packages are enabled and properly configured:
- ✅ **Package creation** should be allowed
- ✅ **Package visibility** can be set to public or private as needed

## Package Visibility

By default, packages published to GHCR inherit the repository's visibility:
- **Public repositories** → Public packages
- **Private repositories** → Private packages

### Making Packages Public

If you want public packages from a private repository:

1. Navigate to the package page: `https://github.com/orgs/techtonic-plates-blog/packages`
2. Select the package (e.g., `posts-service`)
3. Go to **Package settings**
4. Change visibility to **Public**

## postsentication for Container Access

### Public Packages
No postsentication required for pulling public packages:
```bash
docker pull ghcr.io/techtonic-plates-blog/posts-service:latest
```

### Private Packages
Requires postsentication with a Personal Access Token (PAT):

1. Create a PAT with `read:packages` scope
2. Login to GHCR:
```bash
echo $PAT | docker login ghcr.io -u USERNAME --password-stdin
```
3. Pull the package:
```bash
docker pull ghcr.io/techtonic-plates-blog/posts-service:latest
```

## Branch Protection (Recommended)

Navigate to: **Settings → Branches**

Add protection rules for the `main` branch:
- ✅ **Require status checks to pass before merging**
  - Add: `test-builds / build-summary`
- ✅ **Require branches to be up to date before merging**
- ✅ **Require pull request reviews before merging**
- ✅ **Dismiss stale PR reviews when new commits are pushed**

## Secrets Management

The workflows use the built-in `GITHUB_TOKEN`, so no additional secrets are required. However, if you need custom postsentication:

Navigate to: **Settings → Secrets and variables → Actions**

Add repository secrets as needed:
- `GHCR_TOKEN`: Custom GitHub token for package access
- `DOCKER_REGISTRY`: Alternative registry URL
- etc.

## Troubleshooting

### Permission Denied Errors
```
ERROR: permission denied: write_package
```

**Solution**: Verify workflow permissions are set to "Read and write permissions"

### Package Not Found
```
ERROR: pull access denied for ghcr.io/techtonic-plates-blog/posts-service
```

**Solutions**:
1. Check package visibility settings
2. Verify postsentication (for private packages)
3. Ensure package was actually published

### Build Failures
```
ERROR: buildx failed with: error: failed to solve
```

**Solutions**:
1. Check Dockerfile syntax
2. Verify all files referenced in Dockerfile exist
3. Check build context and .dockerignore
4. Review workflow logs for specific error details

## Additional Resources

- [GitHub Packages Documentation](https://docs.github.com/en/packages)
- [Publishing Docker Images](https://docs.github.com/en/actions/publishing-packages/publishing-docker-images)
- [GHCR Documentation](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry)
