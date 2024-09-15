fastchess/fastchess \
  -engine name=SacreDieu cmd=target/release/sacre_dieu \
  -engine name=StashBot cmd=stash-bot \
  -games 2 -rounds 500 \
  -each proto=uci tc=8+0.08 \
  -openings order=random file="sprt/openings.epd" format=epd \
  -randomseed \
  -concurrency 6 \
  -ratinginterval 10 \