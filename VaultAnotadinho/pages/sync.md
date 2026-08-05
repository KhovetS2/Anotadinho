---
title: Pendrive Sync
tags: [docs, sync]
created: 2026-08-04
---

# Sincronização por Pendrive

O Anotadinho **não tem código de sync**. O vault é uma pasta comum.

## Workflow

1. Feche o Anotadinho em ambos os PCs
2. Copie a pasta `vault/` pro pendrive
3. No outro PC, copie do pendrive pro mesmo path local
4. Abra o Anotadinho

## Conflitos

Sem lock entre máquinas, é "último que salva vence".
Para evitar: feche o app antes de copiar.

## Sync via Git (futuro)

```bash
cd ~/anotadinho-vault
git add -A && git commit -m "sync"
git push
```

Conflitos de `.md` (texto) são raros e fáceis de resolver.

## Backup

A própria pasta do vault é o backup.
Sugestão: pendrive (rotação) ou cloud storage (rclone, Syncthing).
