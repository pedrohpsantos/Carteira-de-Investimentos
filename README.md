# RustFolio 🦀

Este projeto é uma aplicação Fullstack de alta performance desenvolvida em Rust para o gerenciamento de uma Carteira de Investimentos. Ele consolida uma API veloz, integração com banco de dados PostgreSQL, autenticação JWT robusta, e renderização de páginas HTML pelo lado do servidor utilizando Askama.

## Arquitetura e Tecnologias

A aplicação é dividida em rotas de API REST e rotas Frontend (Server-Side Rendering). A persistência é gerida via SQLx.

**Tecnologias utilizadas:**
- **Rust** com a edição 2024.
- **Axum**: Framework web para roteamento da API e rotas de frontend.
- **SQLx**: Construtor de queries e executor assíncrono para o PostgreSQL, garantindo queries validadas em tempo de compilação.
- **Askama**: Motor de templates para renderização das views em HTML (`dashboard.html`, `login.html`).
- **jwt-simple**: Geração e validação de tokens JWT para manutenção de sessões.
- **PostgreSQL via Docker**: Banco de dados relacional (via `compose.yml`).

## Backlog / Melhorias Implementadas

A versão atual foi evoluída sobre a base do desafio da DIO com as seguintes melhorias:
1. **Modelagem de Portfólio**: Introdução do conceito de carteiras de investimentos (`portfolios` table) associadas aos usuários. Agora, cada usuário tem sua própria quantidade de cada ativo.
2. **Novos Dados de Investimento**: Adicionado o campo `ticker` (código de negociação) ao modelo global de Ativos (`assets`).
3. **Dashboard Melhorado**: O dashboard foi redesenhado (`dashboard.html`) para listar a carteira de investimentos do usuário, calcular o valor total de cada posição (quantidade * valor unitário) e calcular o **valor total da carteira**.
4. **Formulários Interativos**: Inclusão de formulários HTML no dashboard para o usuário cadastrar novos ativos no sistema global, e adicionar ativos à sua carteira pessoal de investimentos.
5. **Correção de Dependências**: Ajustado `jwt-simple` para utilizar exclusivamente `pure-rust`, evitando problemas de compilação C/C++ no Windows com o `boring-sys`.

## Como Executar a Aplicação

1. Suba o banco de dados PostgreSQL utilizando Docker Compose:
   ```bash
   docker compose up -d
   ```
2. Instale o CLI do SQLx (caso não possua):
   ```bash
   cargo install sqlx-cli
   ```
3. Execute as migrações para criar as tabelas do banco de dados (garanta que o `DATABASE_URL` no seu `.env` esteja correto):
   ```bash
   sqlx migrate run
   ```
4. Execute o projeto com o Cargo:
   ```bash
   cargo run
   ```
5. Acesse a aplicação no seu navegador: `http://localhost:3000`.

## Testes e Qualidade

O projeto utiliza o módulo `tests` integrado nas próprias rotas e com fixtures no SQLx para garantir a estabilidade e previsibilidade.

Como rodar os testes:
```bash
cargo test
```
*Observação: As funções dependentes de banco de dados (`sqlx::test`) usarão a pool de testes gerada automaticamente com o schema do seu diretório de `migrations/`.*

## O que aprendi no desafio
- Estruturação de projetos Rust integrando Back-end (APIs) e SSR (Server-Side Rendering) no mesmo projeto com o **Axum**.
- Como estruturar um fluxo de autenticação robusto usando senhas hasheadas e cookies assinados por JWT em Rust.
- O poder das macros do **SQLx** para validação estática de queries ao banco de dados no momento da compilação.
- O mapeamento e serialização das structs Rust para o motor de templates **Askama** (muito eficiente por gerar Rust puro por baixo dos panos).
