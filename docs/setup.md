# Guia de Configuração

## Configurando o Google Cloud

### 1. Criar Projeto

1. Acesse https://console.cloud.google.com
2. Clique em "Novo Projeto"
3. Dê um nome ao projeto (ex: "RustDriveSync")
4. Clique em "Criar"

### 2. Habilitar Google Drive API

1. No menu lateral, vá em "APIs e Serviços" → "Biblioteca"
2. Procure por "Google Drive API"
3. Clique em "Ativar"

### 3. Configurar Tela de Consentimento

1. Vá em "APIs e Serviços" → "Tela de consentimento OAuth"
2. Selecione "Externo" e clique em "Criar"
3. Preencha:
   - Nome do app: RustDriveSync
   - Email de suporte: seu email
   - Domínio da página inicial: pode deixar em branco
   - Email do desenvolvedor: seu email
4. Clique em "Salvar e continuar"
5. Em "Escopos", adicione:
   - `https://www.googleapis.com/auth/drive.file`
6. Salve e continue

### 4. Criar Credenciais OAuth2

1. Vá em "APIs e Serviços" → "Credenciais"
2. Clique em "+ Criar credenciais" → "ID do cliente OAuth"
3. Tipo de aplicativo: "App para computador"
4. Nome: "RustDriveSync CLI"
5. Clique em "Criar"
6. Baixe o arquivo JSON
7. Renomeie para `credentials.json` e coloque na raiz do projeto

## Configurando o RustDriveSync

### 1. Criar arquivo de configuração

```bash
rustdrivesync config --init
cp config.example.toml config.toml
```

### 2. Editar configuração

Abra `config.toml` e configure:

```toml
[source]
path = "/caminho/para/sua/pasta"  # Mude isso!

[google_drive]
credentials_file = "./credentials.json"
token_file = "./token.json"
target_folder_id = "ID_DA_PASTA_DO_DRIVE"  # Veja como obter abaixo
```

### 3. Obter ID da Pasta do Drive

Opção A - Via URL:
1. Abra o Google Drive no navegador
2. Navegue até a pasta desejada
3. A URL será: `https://drive.google.com/drive/folders/1ABC123xyz`
4. O ID é a parte após `/folders/`: `1ABC123xyz`

Opção B - Usar nome (será criada se não existir):
```toml
[google_drive]
target_folder_name = "Backup"  # Em vez de target_folder_id
```

### 4. Autenticar

```bash
rustdrivesync auth
```

Isso:
1. Abrirá seu navegador
2. Pedirá para autorizar o aplicativo
3. Salvará o token em `token.json`

## Testando

```bash
# Teste simples
rustdrivesync sync --once --dry-run

# Se tudo estiver OK, faça o primeiro sync real
rustdrivesync sync --once
```

## Troubleshooting

### "Token expired"
```bash
rustdrivesync auth  # Re-autenticar
```

### "Credentials not found"
Verifique se o arquivo `credentials.json` existe e está no caminho correto.

### "Folder not found"
Verifique o `target_folder_id` ou use `target_folder_name`.
