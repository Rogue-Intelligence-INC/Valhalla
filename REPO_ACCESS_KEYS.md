# Repository Access Keys - Rogue Intelligence LNC.

## Private Repositories

| Repository | Visibility | Purpose |
|------------|------------|---------|
| `s-wind` | Private | Autonomous Algebraic Discovery System |
| `valhalla-rational` | Private | i128/i128 Exact Arithmetic Framework |
| `valhalla-core` | Private | Unified Physics-Based Processing |
| `yaksha-cipher-gpu` | Private | Quantum-Inspired Encryption Engine |

## Creating Access Tokens

### Option 1: Fine-Grained Personal Access Tokens (Recommended)

1. Visit: https://github.com/settings/tokens?type=beta
2. Click "Generate new token"
3. Configure each token:

#### s-wind-access
```
Name: s-wind-access
Expiration: Custom (365 days recommended)
Repository access: Only select repositories
  → Rogue-Intelligence-INC/s-wind
Permissions:
  Repository permissions:
    - Contents: Read-only
```

#### valhalla-rational-access
```
Name: valhalla-rational-access
Expiration: Custom (365 days recommended)
Repository access: Only select repositories
  → Rogue-Intelligence-INC/valhalla-rational
Permissions:
  Repository permissions:
    - Contents: Read-only
```

#### valhalla-core-access
```
Name: valhalla-core-access
Expiration: Custom (365 days recommended)
Repository access: Only select repositories
  → Rogue-Intelligence-INC/valhalla-core
Permissions:
  Repository permissions:
    - Contents: Read-only
```

#### yaksha-cipher-gpu-access
```
Name: yaksha-cipher-gpu-access
Expiration: Custom (365 days recommended)
Repository access: Only select repositories
  → Rogue-Intelligence-INC/yaksha-cipher-gpu
Permissions:
  Repository permissions:
    - Contents: Read-only
```

### Option 2: SSH Deploy Keys (Generated)

Keys saved to: `.keys/` directory

| Repository | Private Key | Public Key |
|------------|-------------|------------|
| s-wind | `deploy_s-wind` | `deploy_s-wind.pub` |
| valhalla-rational | `deploy_valhalla-rational` | `deploy_valhalla-rational.pub` |
| valhalla-core | `deploy_valhalla-core` | `deploy_valhalla-core.pub` |
| yaksha-cipher-gpu | `deploy_yaksha-cipher-gpu` | `deploy_yaksha-cipher-gpu.pub` |

To add manually:
1. Go to repo Settings → Deploy keys
2. Click "Add deploy key"
3. Paste the public key content
4. Title: "Deploy Key - Read Only"

## Usage

### Clone with token:
```bash
git clone https://<TOKEN>@github.com/Rogue-Intelligence-INC/s-wind.git
```

### Clone with SSH key:
```bash
GIT_SSH_COMMAND="ssh -i ~/.keys/deploy_s-wind" git clone git@github.com:Rogue-Intelligence-INC/s-wind.git
```

---
© 2026 Rogue Intelligence LNC. All Rights Reserved.
