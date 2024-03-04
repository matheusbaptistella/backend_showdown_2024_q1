# phase2_deploy

![alt text](../images/image.png)

![alt text](../images/image2.png)

Tirar os checks do bd aumentou desempenho. ~8000 para ~14000

Parar de converter o fromato da TIMESTAMP na query e fazer isso em codigo ~14000 para 11000

Logica correta para credito de somar caiu de 11000 para 8000

Estabalecer Arc<Mutex> para não ter problemas de concorrêncai no banco de dados resolveu os problemas

![alt text](../images/image3.png)