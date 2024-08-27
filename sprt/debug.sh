cutechess/build/cutechess-cli \
  -engine name=BlueCannonBall cmd=target/release/matrix_mate \
  -engine name=Aspect cmd=target/release/matrix_mate \
  -games 2 -rounds 50000 \
  -pgnout "sprt/pgnout.txt" \
  -each proto=uci tc=8+0.08 stderr=sprt/stderr.txt \
  -openings order=random file="sprt/openings.epd" format=epd \
  -concurrency 6 \
  -ratinginterval 10 \