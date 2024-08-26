cutechess/build/cutechess-cli \
  -engine name=BlueCannonBall cmd=versions/v1_basic \
  -engine name=Aspect cmd=versions/v2_move_ordering \
  -games 2 -rounds 50000 \
  -pgnout "pgnout.txt" \
  -sprt elo0=0 elo1=10 alpha=0.05 beta=0.05 \
  -each proto=uci tc=8+0.08 stderr=stderr.txt \
  -openings order=random file="openings.epd" format=epd \
  -concurrency 6 \
  -ratinginterval 10 \