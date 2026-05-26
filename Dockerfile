# Estágio de Compilação
FROM rust:latest as builder
WORKDIR /usr/src/meu_projeto_p2p
COPY . .
RUN cargo build --release

# Estágio de Execução
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/meu_projeto_p2p/target/release/meu_projeto_p2p /usr/local/bin/p2p_node

# Comando padrão (pode ser subscrito no docker-compose)
ENTRYPOINT ["p2p_node"]