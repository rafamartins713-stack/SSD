
---

#  Manual de Instruções: Sistema de Leilões P2P (Blockchain em Rust)


---

##  1. Requisitos Prévios

* **Docker** e **Docker-Compose** instalados.
* **PowerShell** (Recomendado) ou terminal de sistema.
* Navegador Web (Chrome, Edge ou Firefox).

---

##  2. Como Inicializar a Rede

Para garantir que todos os nós começam sincronizados e sem dados residuais de testes anteriores, utilizamos o script de automação:

1. Abrir o **PowerShell** na pasta raiz do projeto.
2. Executar o comando:
```powershell
.\iniciar_demonstracao.ps1

```


*Este script purga volumes antigos (`docker-compose down -v`) e levanta a rede de 5 nós (Bootstrap, Vendedor e 3 Compradores) de forma isolada.*

---

##  3. Guia do Painel Web (`localhost:9080`)

Aceder ao Dashboard no navegador. A interface adapta-se dinamicamente ao nó que escolheres:

### 3.1. Seleção de Identidade

No menu dropdown, escolhe o teu papel:

* **Bootstrap (Porta 9080):** Monitorização global.
* **Vendedor (Porta 9081):** Acesso exclusivo para criar leilões (`criar.html`) e terminar leilões.
* **Compradores (Portas 9082-9084):** Acesso para licitar em leilões ativos.

### 3.2. Operações de Mercado

* **Atualizar Mercado:** Sincroniza a interface com o estado atual da Blockchain.
* **Licitar:** Inserir um valor superior ao lance atual (assinado via **Ed25519**).
* **Minerar:** Acionar o algoritmo de consenso (*Proof of Work*), registando as transações num bloco validado pelo **SHA-256**.

---

##  4. Auditoria e Validação Técnica

### 4.1. Logs de Mineração (Validação Visual)

Para veremos o "coração" da Blockchain a bater, abrimos um terminal e corremos:

```powershell
docker-compose logs -f

```

Aqui vemos o *Proof of Work* em tempo real: o sistema tenta milhões de combinações (`nonce`) até encontrar um hash que comece com os zeros de dificuldade exigidos.

### 4.2. Test Block (Integridade)

No painel web, clicamos no botão **"🛡️ Test Block"**. O sistema irá auditar a cadeia, verificando se todos os hashes estão encadeados corretamente e confirmando o tamanho total da Blockchain.

---

##  5. Testes de Resiliência (Tolerância a Falhas)

Este sistema é descentralizado. Podemos simular falhas sem quebrar a rede:

1. **Parar um nó:** ```powershell
docker-compose stop comprador-1
```

```

2. **Observar:** A interface desse nó deixará de responder, mas o Vendedor e os outros Compradores continuam a minerar e a transacionar normalmente.
3. **Recuperar:** ```powershell
docker-compose start comprador-1
```


```



---

##  Notas de Operação

* **Segurança:** Todas as transações são assinadas digitalmente. A função `is_signature_valid` rejeita automaticamente qualquer tentativa de fraude ou falsificação.
* **Persistência:** Lembrar que, ao correres o script de inicialização, a cadeia é reiniciada. Isto é ideal para demonstrações, garantindo que começas sempre com um ambiente limpo.
* **Encoding:** Se visualizarmos caracteres estranhos no terminal, executamos no PowerShell: `[Console]::OutputEncoding = [System.Text.Encoding]::UTF8`.