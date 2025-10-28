+/(^/=3 5|%!.\)|&!|1000
# bqn => +´(0=3⊸|⌊5⊸|)⊸/↕1000

# k6  => +/&|/~5 3!\:!1000
# !1000  => range of 0->1000
# 5 3!\: => each left modulo
# ~      => not ie. x -> 0, 0 -> 1
# |/     => fold max
# &      => values from indices
# +/     => sum

# ||1000  => define a monadic train with 1000 as input
# !       => range
# &       => indices ( takes boolean mask as w )
# ()      => closure
# %!.\    => each left modulo
# 3 5|    => train block shorthand ( makes the train temporarily dyadic )
# =       => monadic not ie. n -> 0, 0 -> 1
# ^/      => fold max
# +/      => plus reduce

# - e > mt
#   - mto
#     - mco
#       - m_o: "+"
#       - co: "/"
#     - mb
#       - mtb
#         - mco
#           - m_o: "^"
#           - co: "/"
#         - m_o: "="
#         - dito
#           - ts
#             - i: "3"
#             - i: "5"
#           - dto > dco
#             - dh_o: "%!"
#             - co: ".\\"
#       - d_o: "&"
#     - m_o: "!"
#   - ts > i: "1000"
# - EOI: ""
# 
#  0 -> RVAR(0)
#  1 -> MO(!)
#  2 -> DO(&)
#  3 -> BL(10)
#  6 -> LVAR(1)
#  4 -> CO(.\)
#  5 -> DHO(%!)
#  7 -> MO(=)
#  8 -> CO(/)
#  9 -> MO(^)
# 10 -> CO(/)
# 11 -> MO(+)
# 
#  0 -> Int(1000)
#  1 -> T[Int(3), Int(5)]
