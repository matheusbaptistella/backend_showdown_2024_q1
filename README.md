# backend_showdown_2024_q1
This repo aims at explaining my implementation of a backend for the 2024 Q1 Backend Showdown. This is my first experience at writing a backend, especially one that must adhere to the CPU and memory constraints imposed by the competition. I'll start with the details of my initial version, where I attempt to simply make it work, and then go through some optimizations to improve performance.

LEMBRETES:
* Olhar os includes dos outros participantes
* Usar otimizacoes para a versao release (cargo build --release)
* Atentar-se quando nao ha necessidade de retornar corpo de respostas
* Deixar as imagens do Docker publicas, e nao so locais
* Olhar as performance tips no tutorial do postgres
* Usar RETURNING pra armazenar os indices em uma cache etc
* O fato de ser em centavos talvez precise aumentar o tamanho do integer do bd (?)
* OLhar o arquivo load_test e.g. extrapolar o limite de caracteres da descricao -> 422
* Olhar o twitter com dicas de otimizacao
* Talvez fazer algumas coisas antes de mandar pro banco reduza o gargalo e.g. o timestamp
* Talvez tirar o default 0
* Testar criar structs parra serializar os dados