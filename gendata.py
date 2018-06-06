#!/usr/bin/env python

import sys
from time import sleep
from random import randint
from math import sin, cos, floor, atan2, asin, acos

def log(chan, *args):
	rslt = "~."+str(chan)+'@'
	for i in args:
		rslt+=str(i)+','
	print rslt


if __name__ == '__main__':
	test = 0
	try:
		test = int(sys.argv[1])
	except:
		pass

	if test == 0:
		data = """
		~.Bar@-0.5,-0.5, 0.2
		~.Bar@0.0,0.5, 1.5
		~.Bar@0.5,-0.25, 1.0
		"""
		for i in data.split('\n'):
			print i
			# sleep(randint(0,2))

	elif test == 1:
		res = 10.0
		count = 5#300
		for i in xrange(0, int(count*2*3.14159*res)):
			y = sin(i/res)
			y = y*min(floor(abs(y)+.2), 1)
			# print ":#D:"+str(i)+","+str(y)
			log("d", i, y)
			log("e", i, sin(((i+0)/res))*10)
			sys.stdout.flush()
			# sleep(0.08)

	elif test == 2:
		r = .999
		divs = 64/4 + 3
		step = r/divs
		for x in xrange(divs): 
			log("A", 1*r, 1*r)
			log("A", 1*r, -1*r)
			log("A", -1*r, -1*r)
			log("A", -1*r, 1*r)

			log("B", 1*r, 1*r)
			log("B", 1*r, -1*r)
			log("B", -1*r, -1*r)
			log("B", -1*r, 1*r)
			r -= step
			sys.stdout.flush()
			# sleep(.5)

	elif test == 3:
		divs = 100
		step = 2.0/divs
		sigs = "abcdefghij"
		maxx = 0
		maxy = 0
		for s in sigs:
			log(s, 0, -2)
		for i in xrange(divs): 
			pos = 0
			for s in sigs:
				x = i*step
				if x > 2.0*(pos/float(len(sigs))):
					sin = 1#2*(pos % 2)-1
					y = sin*(x - 2.0*(pos/float(len(sigs))))
				else:
					y = 0
				log(s, x, y)
				pos += 1
				if x > maxx:
					maxx = x
				if y > maxy:
					maxy = y
		for s in sigs:
			log(s, maxx, maxy)

	elif test == 4:
		divs = 100
		step = 2*3.14159 / divs
		theta = 0
		textheight = 0.05

		y1 = lambda t: sin(t) - textheight if sin(t) - textheight > -1 else sin(t) + textheight
		t1 = lambda y: asin(y)

		t0 = 0
		for i in xrange(divs):
			# theta += 0.2
			# log("circle", cos(theta), sin(theta))
			t = step*i

			# log("y1", t, y1(t));
			# log("t1", t, t1(y1(t)))

			log("rads", t, t0)
			y = y1(t0)
			print y
			t0 = t1(y)

	elif test == 5:
		#Requires that all channels are bound together 
		from itertools import combinations, islice
		step = 2/26.0
		pos = -1+ step /2
		cp = pos
		cpy = pos
		alphabet = list("abcdefghijklmnopqrstuvwxyz")
		words = ["".join(x) for x in islice(combinations(alphabet, 3), 500)]
		for i,c in enumerate(alphabet + range(0,150) + words):
			log(c, cp, cpy, 30)
			sys.stdout.flush()
			cp += step
			if cp > 1:
				cpy += step
				cp = pos
			# sleep(0.1)
