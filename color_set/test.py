from random import uniform
from math import floor

def hsvToRGB(h, s, v):
    """Convert HSV color space to RGB color space
    
    @param h: Hue
    @param s: Saturation
    @param v: Value
    return (r, g, b)  
    """
    hi = floor(h / 60.0) % 6
    f =  (h / 60.0) - floor(h / 60.0)
    p = v * (1.0 - s)
    q = v * (1.0 - (f*s))
    t = v * (1.0 - ((1.0 - f) * s))
    return {
        0: (v, t, p),
        1: (q, v, p),
        2: (p, v, t),
        3: (p, q, v),
        4: (t, p, v),
        5: (v, p, q),
    }[hi]


theta = .531#uniform(0,1)
def next():
    # use golden ratio
    global theta
    golden_ratio_conjugate = 0.618033988749895
    theta += golden_ratio_conjugate
    theta %= 1
    return hsvToRGB(theta*360., .9, .9)


for i in xrange(100):
    print next()
