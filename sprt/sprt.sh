cutechess/build/cutechess-cli \
  -engine name=Aspect cmd=releases/v2_move_ordering \
  -engine name=BlueCannonBall cmd=releases/v1_basic \
  -games 2 -rounds 50000 \
  -pgnout "sprt/pgnout.txt" \
  -sprt elo0=0 elo1=10 alpha=0.05 beta=0.05 \
  -each proto=uci tc=8+0.08 stderr=sprt/stderr.txt \
  -openings order=random file="sprt/openings.epd" format=epd \
  -concurrency 6 \
  -ratinginterval 10 \