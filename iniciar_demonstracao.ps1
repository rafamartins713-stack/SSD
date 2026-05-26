# 1. Parar, limpar volumes persistentes antigos e relançar
Write-Host " A limpar volumes e a recriar os nós de raiz..." -ForegroundColor Cyan
docker-compose down --volumes --remove-orphans
docker-compose up -d --build
Start-Sleep -Seconds 10
# 2. Monitorização Ativa do Arranque
Write-Host " A verificar prontidão da API Rust (Porta 9080)..." -ForegroundColor Yellow
$max_tentativas = 40
$tentativa = 0
$online = $false

while (-not $online -and $tentativa -lt $max_tentativas) {
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:9080/api/auctions" -Method Get -TimeoutSec 2 -ErrorAction Stop -UseBasicParsing
        if ($response.StatusCode -eq 200) { $online = $true }
    } catch {
        $tentativa++
        Start-Sleep -Seconds 3
        Write-Host "   [Tentativa $tentativa/$max_tentativas] A compilar código corrigido..." -ForegroundColor Gray
    }
}

if (-not $online) {
    Write-Host " Erro fatal: Executa 'docker-compose logs bootstrap' para ver o erro." -ForegroundColor Red
    exit
}

# 3. Injetar Leilões Automáticos
Write-Host " API Pronta! A propagar leilões..." -ForegroundColor Cyan

$body1 = @{ item = "Caneta_Universitaria"; min_bid = 10 } | ConvertTo-Json
$res1 = Invoke-RestMethod -Uri "http://localhost:9080/api/auctions" -Method Post -Body $body1 -ContentType "application/json"
Write-Host "   -> Leilão 1 Criado: $($res1.auction_id)" -ForegroundColor Gray

$body2 = @{ item = "Caderno_Rust_Pro"; min_bid = 25 } | ConvertTo-Json
$res2 = Invoke-RestMethod -Uri "http://localhost:9080/api/auctions" -Method Post -Body $body2 -ContentType "application/json"
Write-Host "   -> Leilão 2 Criado: $($res2.auction_id)" -ForegroundColor Gray

$body3 = @{ item = "livro_Pro"; min_bid = 30 } | ConvertTo-Json
$res3 = Invoke-RestMethod -Uri "http://localhost:9080/api/auctions" -Method Post -Body $body3 -ContentType "application/json"
Write-Host "   -> Leilão 3 Criado: $($res2.auction_id)" -ForegroundColor Gray

# 4. Mine Automático para fixar e persistir os dados na Blockchain
Write-Host " A forçar mineração do Bloco Genesis + Leilões..." -ForegroundColor Yellow
Start-Sleep -Seconds 2
$mine_res = Invoke-RestMethod -Uri "http://localhost:9080/api/mine" -Method Post -ContentType "application/json"
Write-Host "   -> Bloco Minerado com Sucesso: Bloco #$($mine_res.block_index)" -ForegroundColor Green

Write-Host " Concluído com Sucesso! Atualiza o teu index.html." -ForegroundColor Green