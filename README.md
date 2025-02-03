# Geladeira Inteligente

Este projeto implementa um sistema de monitoramento e controle para uma geladeira inteligente, desenvolvido em Rust. Ele utiliza a biblioteca Tokio para comunicação assíncrona via TCP e um protocolo de mensagens baseado no formato HTTP para a troca de informações entre sensores, atuadores e clientes.

## Requisitos

Antes de executar o projeto, certifique-se de ter os seguintes requisitos instalados:

- [Rust](https://www.rust-lang.org/) (versão estável mais recente)
- Cargo (gerenciador de pacotes do Rust, incluído na instalação do Rust)
- [Tokio](https://tokio.rs/) (biblioteca para operações assíncronas)

## Instalação

Clone o repositório e navegue até a pasta do projeto:

```bash
git clone https://github.com/contrasov/geladeira-inteligente.git
cd geladeira-inteligente
```

## Como Rodar o Projeto

1. Compile o projeto:
   ```bash
   cargo build --release
   ```

2. Execute o servidor:
   ```bash
   cargo run
   ```

O servidor será iniciado na porta `8080` e aguardará conexões dos sensores, atuadores e clientes.

## Estrutura das Mensagens

O protocolo de comunicação segue um formato semelhante ao HTTP. Exemplos de comandos suportados:

- **Identificação de sensores e atuadores**:
  ```
  SENSOR/1.0 IDENTIFY ID <uuid>
  ```
  Resposta esperada:
  ```
  GERENCIADOR/1.0 200 OK
  ```

- **Solicitar status do sistema**:
  ```
  CLIENT/1.0 GET_STATUS
  ```
  Resposta esperada:
  ```
  GERENCIADOR/1.0 200 OK
  TEMPERATURA: 4.5 | ID: <uuid>
  PORTA: FECHADA | ID: <uuid>
  ESTOQUE: 75% | ID: <uuid>
  REFRIGERADOR: LIGADO | ID: <uuid>
  LUZ: APAGADA | ID: <uuid>
  ALARME: NORMAL
  ```

- **Definir temperatura ideal**:
  ```
  CLIENT/1.0 SET_LIMITE 3.0
  ```
  Resposta esperada:
  ```
  GERENCIADOR/1.0 200 OK
  LIMITE: 3.0
  ```

- **Alterar estado da porta**:
  ```
  CLIENT/1.0 SET_PORTA ABERTA
  ```
  Resposta esperada:
  ```
  GERENCIADOR/1.0 200 OK
  PORTA: ABERTA
  ```


