# LLM Provider Setup

How to configure the Anthropic API key for ForgeMaster agent runtime pods.

## How It Works

```
.env.forgemaster (actual API key, gitignored)
  + local.env (secret name/key config, checked in)
    → Ansible creates K8s Secret in forgemaster-system
      → PendingStrategy copies Secret to task namespace
        → runtime pod reads ANTHROPIC_API_KEY from secret
```

## Step 1: Get Your API Key

1. Go to [console.anthropic.com](https://console.anthropic.com)
2. Navigate to **API Keys**
3. Create a new key or copy an existing one

## Step 2: Create .env.forgemaster

Create `.env.forgemaster` in the project root (gitignored):

```env
LLM_PROVIDER_API_KEY=sk-ant-your-key-here
```

This file holds the actual secret value. It is never committed to git.

## Step 3: Configure local.env (optional)

The file `ansible/environments/default/local.env` controls the K8s Secret name and key:

```env
LLM_PROVIDER_SECRET_NAME=anthropic-credentials
LLM_PROVIDER_SECRET_KEY=api-key
```

The defaults work for most setups. Change only if you use a different secret structure.

## Step 4: Deploy

```bash
make cluster
```

This:
1. Sources `.env.forgemaster` (actual key) and `local.env` (secret name/key config)
2. Ansible creates the K8s Secret `anthropic-credentials` in `forgemaster-system`
3. Deploys all services

When a task runs, the Agent Controller automatically copies the secret from `forgemaster-system` to the task namespace before creating the runtime pod.

## Verification

Check the secret exists:

```bash
kubectl get secret anthropic-credentials -n forgemaster-system
```

After a task runs, verify it was copied to the task namespace:

```bash
kubectl get secret anthropic-credentials -n task-XXXXX
```

Check that a runtime pod picks up the secret:

```bash
kubectl describe pod orchestrator-task-XXXXX -n task-XXXXX
```

Look for:

```
ANTHROPIC_API_KEY: <set to the key 'api-key' in secret 'anthropic-credentials'>
```

## Different Environments

For a different environment (e.g. staging), create `ansible/environments/staging/local.env`:

```env
LLM_PROVIDER_SECRET_NAME=anthropic-credentials-staging
LLM_PROVIDER_SECRET_KEY=api-key
```

Then deploy with:

```bash
make cluster ENV=staging
```

The `.env.forgemaster` file is shared across environments (same API key). Use separate `.env.forgemaster` files per machine if needed.
