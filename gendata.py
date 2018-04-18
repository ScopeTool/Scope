#!/usr/bin/env python

import sys
from time import sleep
from random import randint
from math import sin, floor

def log(x,y, chan = 'A'):
	print "~."+chan+"@"+str(x)+","+str(y)

data = """
~.Ca@-0.5,-0.5, 0.2
~.Ca@0.0,0.5, 1.5
~.Ca@0.5,-0.25, 1.0
"""
# :#C:1,1
# :#C:.5,.2
# :#C:.5,1.0
# :#C:0,0
# :#C:0,0
for i in data.split('\n'):
	print i
	# sleep(randint(0,2))

# res = 10.0
# for i in xrange(0, int(300*2*3.14159*res)):
# 	y = sin(i/res)
# 	y = y*min(floor(abs(y)+.2), 1)
# 	# print ":#D:"+str(i)+","+str(y)
# 	print ":#d:"+str(y)
# 	print ":#e:"+str(sin(((i+0)/res))*10)
# 	sys.stdout.flush()
# 	sleep(.05)

r = .9
divs = 64/4 + 3
step = r/divs
for x in xrange(divs):
	log(1*r, 1*r)
	log(1*r, -1*r)
	log(-1*r, -1*r)
	log(-1*r, 1*r)
	r -= step