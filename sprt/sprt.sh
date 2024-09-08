fastchess/fastchess \
  -engine name=Aspect cmd=releases/v13_see \
  -engine name=BlueGarbageBall cmd=releases/v12_history_heuristic \
  -games 2 -rounds 50000 \
  -pgnout "sprt/pgnout.txt" \
  -sprt elo0=0 elo1=10 alpha=0.05 beta=0.05 \
  -each proto=uci tc=5+0.05 \
  -openings order=random file="sprt/openings.epd" format=epd \
  -randomseed \
  -concurrency 6 \
  -ratinginterval 10 \
  # -config file="config.json" \