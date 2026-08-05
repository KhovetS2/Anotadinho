# Pendrive Sync

O Anotadinho **não tem código de sync**. O vault é uma pasta comum no
filesystem, e o usuário é responsável por movê-la entre máquinas.

## Workflow

### Setup inicial
1. Crie uma pasta `vault/` em algum lugar (HD, SSD, pendrive)
2. Dentro dela, crie `pages/`, `journals/`, `assets/`
3. No Anotadinho, abra essa pasta como vault

### Sincronização diária

**Entre Fedora (pessoal) e WSL (trabalho):**

1. Feche o Anotadinho em ambos
2. Copie a pasta `vault/` pro pendrive
3. No outro PC, copie do pendrive pro mesmo path local
4. Abra o Anotadinho

**Conflitos (edições simultâneas):**
- Sem lock entre máquinas, é "último que salva vence"
- Para evitar: feche o app antes de copiar
- Ou use Git no vault (próximo ciclo adiciona suporte opcional)

## Sync via Git (futuro)

O Anotadinho tem lock files em `vault/.anotadinho/locks/` que
servem pra multi-instância **na mesma máquina**. Para sync entre
máquinas, Git é a melhor opção:

```bash
# No Fedora
cd ~/anotadinho-vault
git add -A
git commit -m "sync $(date)"
git push

# No WSL (depois de copiar)
cd ~/anotadinho-vault
git pull
```

Conflitos de `.md` (texto) são raros e fáceis de resolver manualmente.

## Backup

A pasta do vault é o backup. Não precisa de mais nada. Sugestão:

- Pendrive (rotação)
- Ou: cloud storage (rclone, Syncthing) - fora do escopo do app
