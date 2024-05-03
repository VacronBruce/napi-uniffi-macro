import unittest

import hello

class TestHello(unittest.TestCase):
    def test_sum(self):
        self.assertEqual(hello.sum(3, 4), 7)
    def test_enum_week(self):
        self.assertEqual(hello.Week.SUN._value_, 6)
    def test_fn_week(self):
        self.assertEqual(hello.get_week_value(hello.Week.SAT), 5)
    def test_service_hello(self):
        s = hello.service_new()
        self.assertEqual(hello.service_hello(s), "SERVICE:HELLO")

if __name__ == '__main__':
    unittest.main() 