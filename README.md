# backend_showdown_2024_q1
This repo aims to explain my implementation of a backend for the 2024 Q1 Backend Showdown. This is my first experience writing a backend, especially one that must adhere to the CPU and memory constraints imposed by the competition. I'll separate my api's development into modules so that I can cover the steps that I took. Initially, I'll explain my logic and some concepts to develop the core of the api, to understand the Rust libraries involved in backend development. Then, how to deploy the code and include the use of a load balancer. Finally, some optimizations to improve performance.

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