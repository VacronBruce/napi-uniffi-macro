import unittest

import hello

class TestHello(unittest.TestCase):
    def test_sum(self):
        print("Start test sum")
        self.assertEqual(hello.sum(3, 4), 7)

if __name__ == '__main__':
    unittest.main() 