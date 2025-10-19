# Conversor de Máquinas de Turing — Modelos Sipser e Infinito

Este projeto implementa um **conversor entre dois modelos de Máquinas de Turing**:

- **Modelo Infinito → Modelo Sipser**
- **Modelo Sipser → Modelo Infinito**

O programa lê um arquivo de entrada (`.in`) descrevendo uma máquina de Turing em um dos modelos e gera automaticamente um arquivo de saída (`.out`) contendo a máquina equivalente no outro modelo.

---

## 🧩 Estrutura do Projeto

├── src/
│ └── main.rs # Código-fonte principal (este arquivo)
├── example.in # Exemplo de arquivo de entrada (opcional)
└── Cargo.toml # Arquivo de configuração do projeto Rust

---

## ⚙️ Pré-requisitos

Para compilar e executar o programa, é necessário ter o **Rust** instalado.

### 🔸 Instalar Rust (caso ainda não tenha)

A forma mais simples e recomendada é usando o **Rustup**, o gerenciador de versões do Rust.

Execute no terminal:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
Em seguida, reinicie o terminal e verifique a instalação:

```bash
rustc --version
cargo --version
```
Se ambos os comandos retornarem versões válidas, o Rust foi instalado com sucesso.

## 🧪 Compilação e Execução
1. Clonar ou baixar o projeto
```bash
git clone https://github.com/lviana15/turing-simulator
cd turing-simulator
```
(ou simplesmente coloque o arquivo main.rs dentro de uma pasta com um Cargo.toml válido)

2. Executar o programa
O programa aceita um argumento opcional: o arquivo de entrada (.in).

✅ Exemplo de uso:
```bash
cargo run -- exemplo.in
```
Se o arquivo não for especificado, ele usará por padrão `example.in`
```bash
cargo run
```

## 📥 Formato do Arquivo de Entrada
O arquivo de entrada deve começar com um cabeçalho identificando o tipo de máquina, seguido pelas transições:

;I → Máquina Infinita

;S → Máquina Sipser

Cada linha de transição tem o formato:

```txt
<estado_atual> <símbolo_lido> <símbolo_escrito> <direção> <novo_estado>
```

Exemplo:
```txt
;I
0 0 1 r 1
1 1 1 l 0
```
Obs: O programa sempre considera o estado `0` como estado inicial
O programa criará automaticamente um arquivo de saída com a mesma base do nome, mas extensão .out.

## 📤 Saída
Ao executar o programa, ele exibirá algo como:
```txt
✅ Successfully converted to Sipser model.
 Input: example.in
 Output: example.out
```
E o arquivo example.out conterá a máquina equivalente no outro modelo.

## ⚠️ Erros Comuns
"Input file name must end with '.in'"
→ Certifique-se de que o arquivo de entrada tem a extensão .in.

"Invalid machine type header"
→ O cabeçalho do arquivo deve ser ;I ou ;S.

"Failed to parse transition line"
→ Verifique se as linhas de transição estão bem formatadas e contêm 5 partes.
