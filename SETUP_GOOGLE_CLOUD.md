# 🔧 Guia Completo: Configurar Google Cloud Console para RustDriveSync

Este guia detalha **passo a passo** como obter as credenciais OAuth 2.0 necessárias para usar o RustDriveSync.

⏱️ **Tempo estimado**: 10-15 minutos  
💰 **Custo**: Gratuito (não requer billing/cartão de crédito)

---

## 📋 Visão Geral

Para sincronizar com o Google Drive, você precisa:
1. ✅ Criar um projeto no Google Cloud
2. ✅ Habilitar a Google Drive API
3. ✅ Configurar OAuth Consent Screen
4. ✅ Criar credenciais OAuth 2.0 (Desktop App)
5. ✅ Baixar o arquivo `credentials.json`

---

## 🚀 Passo a Passo Detalhado

### Passo 1: Criar um Projeto no Google Cloud Console

#### 1.1 Acessar o Console

1. Abra seu navegador
2. Acesse: **https://console.cloud.google.com**
3. Faça login com sua conta Google
4. Aceite os termos de serviço se for a primeira vez

#### 1.2 Criar Novo Projeto

1. **Localize o seletor de projeto**
   - No topo da página, ao lado do logo "Google Cloud"
   - Você verá um dropdown com o texto do projeto atual ou "Select a project"

2. **Clique no seletor de projeto**
   - Uma janela modal abrirá mostrando seus projetos existentes

3. **Clique em "NEW PROJECT" (Novo Projeto)**
   - Botão azul no canto superior direito da modal

4. **Preencha os dados do projeto**
   - **Project name**: Digite "RustDriveSync" (ou qualquer nome de sua preferência)
   - **Organization**: Deixe como está (No organization)
   - **Location**: Deixe como está (No organization)

5. **Clique em "CREATE" (Criar)**
   - Aguarde 5-10 segundos enquanto o projeto é criado
   - Você verá uma notificação quando estiver pronto

6. **Selecione o projeto criado**
   - Clique novamente no seletor de projeto no topo
   - Selecione "RustDriveSync" da lista

**✅ Checkpoint**: No topo da página deve aparecer "RustDriveSync" no seletor de projeto

---

### Passo 2: Habilitar a Google Drive API

#### 2.1 Acessar a API Library

1. **Abrir o menu lateral**
   - Clique no ícone ☰ (hamburguer) no canto superior esquerdo

2. **Navegar para APIs & Services**
   - No menu lateral, procure por "APIs & Services"
   - Passe o mouse sobre ele
   - Clique em **"Library"**

   **Ou acesse diretamente**: https://console.cloud.google.com/apis/library

#### 2.2 Buscar e Habilitar a API

1. **Buscar pela API**
   - Na barra de busca no topo, digite: `Google Drive API`
   - Pressione Enter

2. **Selecionar a API correta**
   - Clique em **"Google Drive API"** (ícone azul com logo do Drive)
   - Certifique-se que é o oficial do Google

3. **Habilitar a API**
   - Clique no botão azul **"ENABLE"** (Habilitar)
   - Aguarde 2-3 segundos
   - Você será redirecionado para a página da API

**✅ Checkpoint**: Você verá "API enabled" e um dashboard com métricas (ainda vazias)

---

### Passo 3: Configurar OAuth Consent Screen

⚠️ **IMPORTANTE**: Este passo é obrigatório antes de criar credenciais. Pular este passo resultará em erro!

#### 3.1 Acessar OAuth Consent Screen

1. **No menu lateral**
   - Clique em ☰ (menu hamburguer)
   - Vá em: **"APIs & Services"** → **"OAuth consent screen"**

   **Ou acesse diretamente**: https://console.cloud.google.com/apis/credentials/consent

#### 3.2 Escolher Tipo de Usuário

1. **Selecione "External"**
   - Radio button ao lado de "External"
   - Necessário para uso pessoal (não empresarial)

2. **Clique em "CREATE"**

#### 3.3 Configurar App Information (Página 1/4)

**Campos Obrigatórios** (marcados com *):

1. **App name**: `RustDriveSync` (ou seu nome preferido)
2. **User support email**: Selecione seu email do dropdown
3. **App logo**: (Opcional) - Deixe em branco
4. **App domain**: (Opcional) - Deixe em branco
5. **Authorized domains**: (Opcional) - Deixe em branco
6. **Developer contact information**: Digite seu email

**Clique em "SAVE AND CONTINUE"** (botão azul no fim da página)

#### 3.4 Configurar Scopes (Página 2/4)

1. **Clique em "ADD OR REMOVE SCOPES"**

2. **Na modal que abrir, busque por**: `drive.file`

3. **Selecione o scope**:
   - ✅ `.../auth/drive.file` - "View and manage Google Drive files and folders that you have opened or created with this app"
   - **NÃO** selecione `.../auth/drive` completo (muito permissivo)

4. **Clique em "UPDATE"**

5. **Verifique que o scope aparece na tabela**
   - Você deve ver 1 scope selecionado

6. **Clique em "SAVE AND CONTINUE"**

#### 3.5 Adicionar Test Users (Página 3/4)

⚠️ **Crucial**: Sem test users, você não conseguirá autenticar!

1. **Clique em "+ ADD USERS"**

2. **Digite seu email**
   - O mesmo email que você usará para sincronizar
   - Pode adicionar até 100 test users

3. **Clique em "ADD"**

4. **Verifique que seu email aparece na lista**

5. **Clique em "SAVE AND CONTINUE"**

#### 3.6 Revisar e Confirmar (Página 4/4)

1. **Revise as informações**
   - App name: RustDriveSync ✓
   - Scopes: .../auth/drive.file ✓
   - Test users: seu@email.com ✓

2. **Clique em "BACK TO DASHBOARD"**

**✅ Checkpoint**: Você deve ver "Publishing status: Testing" no dashboard

---

### Passo 4: Criar Credenciais OAuth 2.0

#### 4.1 Acessar Credentials

1. **No menu lateral**
   - ☰ → **"APIs & Services"** → **"Credentials"**

   **Ou acesse**: https://console.cloud.google.com/apis/credentials

#### 4.2 Criar Nova Credencial

1. **Clique em "+ CREATE CREDENTIALS"** (topo da página)

2. **Selecione "OAuth client ID"** do dropdown

#### 4.3 Configurar o Client ID

Se aparecer aviso "To create an OAuth client ID, you must first configure your consent screen":
- Você pulou o Passo 3! Volte e complete-o.

**Preencha**:

1. **Application type**: Selecione **"Desktop app"** do dropdown
   - ⚠️ NÃO selecione "Web application" ou "Service account"

2. **Name**: `RustDriveSync CLI` (ou qualquer nome descritivo)

3. **Clique em "CREATE"**

#### 4.4 Baixar o arquivo JSON

1. **Popup "OAuth client created" aparecerá**
   - Mostra Client ID e Client Secret

2. **Clique em "DOWNLOAD JSON"** (ícone ⬇️)
   - Arquivo será baixado como: `client_secret_XXXXX.apps.googleusercontent.com.json`
   - Onde XXXXX é um ID único

3. **Clique em "OK"** para fechar o popup

**✅ Checkpoint**: Você tem um arquivo `.json` na pasta Downloads

---

### Passo 5: Organizar e Proteger Credenciais

#### 5.1 Renomear e Mover o Arquivo

**Linux/macOS**:
```bash
# Criar pasta de configuração
mkdir -p ~/.config/rustdrivesync

# Mover e renomear (ajuste o nome do arquivo conforme necessário)
mv ~/Downloads/client_secret_*.json ~/.config/rustdrivesync/credentials.json

# Proteger com permissões restritas
chmod 600 ~/.config/rustdrivesync/credentials.json

# Verificar
ls -la ~/.config/rustdrivesync/credentials.json
```

**Windows (PowerShell)**:
```powershell
# Criar pasta de configuração
New-Item -Path "$env:APPDATA\rustdrivesync" -ItemType Directory -Force

# Mover e renomear
Move-Item "$env:USERPROFILE\Downloads\client_secret_*.json" "$env:APPDATA\rustdrivesync\credentials.json"

# Verificar
Get-Item "$env:APPDATA\rustdrivesync\credentials.json"
```

#### 5.2 Verificar Conteúdo do Arquivo

O arquivo `credentials.json` deve ter esta estrutura:

```json
{
  "installed": {
    "client_id": "XXXXX.apps.googleusercontent.com",
    "project_id": "rustdrivesync-XXXXX",
    "auth_uri": "https://accounts.google.com/o/oauth2/auth",
    "token_uri": "https://oauth2.googleapis.com/token",
    "auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs",
    "client_secret": "XXXXX-XXXXX",
    "redirect_uris": ["http://localhost", "urn:ietf:wg:oauth:2.0:oob"]
  }
}
```

Se estiver diferente (ex: tem chave `web` ao invés de `installed`):
- ❌ Você criou credenciais do tipo errado
- ✅ Volte ao Passo 4.3 e crie como "Desktop app"

---

## 🎉 Pronto! Próximos Passos

Agora que você tem o `credentials.json`:

### 1. Configure o RustDriveSync

```bash
# Criar config
rustdrivesync config --init

# Editar config
cp config.example.toml config.toml
vim config.toml  # ou seu editor preferido
```

No `config.toml`, configure:
```toml
[google_drive]
credentials_file = "~/.config/rustdrivesync/credentials.json"  # Linux/macOS
# credentials_file = "%APPDATA%/rustdrivesync/credentials.json"  # Windows
```

### 2. Autenticar

```bash
rustdrivesync auth
```

Isso irá:
1. Abrir seu navegador
2. Pedir para você fazer login no Google
3. Mostrar aviso "Google hasn't verified this app" - clique em "Advanced" → "Go to RustDriveSync (unsafe)"
4. Aprovar as permissões
5. Salvar o token de acesso em `token.json`

### 3. Sincronizar

```bash
rustdrivesync sync
```

---

## ❓ Troubleshooting

### Erro: "Access blocked: This app's request is invalid"

**Causa**: OAuth Consent Screen não configurado ou incompleto

**Solução**:
1. Volte ao Passo 3
2. Certifique-se de configurar todas as 4 páginas
3. Adicione seu email como Test User
4. Tente autenticar novamente

### Erro: "The OAuth client was not found"

**Causa**: Credenciais do tipo errado (Web app ou Service Account)

**Solução**:
1. Volte ao Passo 4
2. Delete a credencial existente
3. Crie nova do tipo **"Desktop app"**
4. Baixe o novo `credentials.json`

### Erro: "invalid_grant" ou "Token has been expired or revoked"

**Causa**: Token antigo ou inválido

**Solução**:
```bash
# Deletar token antigo
rm ~/.config/rustdrivesync/token.json  # Linux/macOS
# del %APPDATA%\rustdrivesync\token.json  # Windows

# Autenticar novamente
rustdrivesync auth
```

### Aviso: "Google hasn't verified this app"

**Causa**: Aplicação em modo "Testing"

**Isso é normal!** Para uso pessoal, não precisa publicar o app.

**O que fazer**:
1. Clique em "Advanced"
2. Clique em "Go to RustDriveSync (unsafe)"
3. Aprovar as permissões

O app é seguro - você mesmo criou!

### Não consigo baixar o credentials.json

**Solução 1** - Via popup:
1. Vá em: APIs & Services → Credentials
2. Clique em "+ CREATE CREDENTIALS" novamente
3. Baixe quando o popup aparecer

**Solução 2** - Via lista:
1. Vá em: APIs & Services → Credentials
2. Na seção "OAuth 2.0 Client IDs", localize sua credencial
3. Clique no nome (ex: "RustDriveSync CLI")
4. Clique no ícone de download ⬇️ no topo direito

---

## 🔐 Segurança

### Proteção de Credenciais

- ✅ **NUNCA** commite `credentials.json` no Git
- ✅ Adicione ao `.gitignore`:
  ```
  credentials.json
  token.json
  ```
- ✅ Use permissões restritas: `chmod 600`
- ✅ Não compartilhe o arquivo com ninguém

### Escopos OAuth

O RustDriveSync usa apenas:
- `https://www.googleapis.com/auth/drive.file`

Isso significa que o app só pode:
- ✅ Ver e gerenciar arquivos QUE ELE MESMO criou
- ❌ NÃO pode ver outros arquivos no seu Drive
- ❌ NÃO pode deletar arquivos que não criou

**Seguro e com permissões mínimas!**

### Revogar Acesso

Se quiser remover acesso do RustDriveSync:

1. Acesse: https://myaccount.google.com/permissions
2. Encontre "RustDriveSync"
3. Clique em "Remove Access"

---

## 📚 Links Úteis

- **Google Cloud Console**: https://console.cloud.google.com
- **OAuth 2.0 Docs**: https://developers.google.com/identity/protocols/oauth2/native-app
- **Drive API Docs**: https://developers.google.com/drive/api/guides/about-sdk
- **Manage API Access**: https://myaccount.google.com/permissions

---

## ✅ Checklist Final

Antes de prosseguir, verifique:

- [ ] Projeto criado no Google Cloud Console
- [ ] Google Drive API habilitada
- [ ] OAuth Consent Screen configurado (4 páginas completas)
- [ ] Test User adicionado (seu email)
- [ ] Credenciais OAuth 2.0 criadas (Desktop app)
- [ ] Arquivo `credentials.json` baixado
- [ ] Arquivo renomeado e movido para pasta correta
- [ ] Permissões de arquivo configuradas (600)
- [ ] Arquivo tem estrutura JSON correta (chave `installed`)

Se todos os itens estiverem ✅, você está pronto para usar o RustDriveSync!

---

**Dúvidas?** Abra uma issue: https://github.com/chapzin/rustdrivesync/issues

**Documentação completa**: [README.md](README.md) | [GUIA_DE_USO.md](GUIA_DE_USO.md)
