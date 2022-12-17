Testing
=======

Automated Testing Goals
-----------------------
Tests should be:

1) Easy to write
2) Easy to read
3) Easy to maintain
4) Easy to understand when they go wrong

Testing Conventions
-------------------
Towards these goals our team has adopted a set of conventions across all our code bases. These conventions make it much easier for someone familiar with our code base but unfamiliar with a particular component to quickly jump in, understand the code, and be productive writing code and updating/creating tests.

* each unit test should test exactly one thing. This means that usually you will only have one expect per test. You might have additional expects if you are trying to show, say, that something is true before the code under test is called, and false after. But if you are trying to show that an exception to the base case is different from the base case, the base case should be tested, then the exception should be tested in two different unit tests.
* Unit tests should tell a story. They should start with general functionality/base cases, and move to special cases, edge cases, and error states. When adding tests to an existing test suite, it is particularly important that you understand the "story" and insert your new tests in an appropriate location.
* The test method name describes what, exactly is being tested. Method name is used rather docstring because experience shows that on average more programmers will update a method name when the test changes than will update a docstring, thus resulting in a higher likelihood of correct test description over time. Use the following convention for test method name:

```
# def test_<method/code under test>_where_<conditions/inputs/state>_expect_<result>

e.g.:
def test_multiplication_where_both_numbers_positive_expect_positive_number():
    ...

def test_division_where_divisor_is_zero_expect_divide_by_zero_error():
    ...
```

* Minimize whenever possible the use of implicit framework setup/teardown functions. When setup is in the same method as the test, you can look at a test in total isolation and understand it. If you have complex, unavoidable setup, write a function with a descriptive name that is called at the beginning of each test. Explicitness is our friend. An exception might be for resetting a database or other housekeeping that is entirely unneeded to understand the test code.
* The structure should be as follows:
    * setup
    * nut/eut = node/element under test (optional, when there is a particular xml node you are testing
    * cut = code under test (the method that this unit test actually tests
    * actual = cut(...)
    * expected = xxx
    * assertion(s)
* xml should include the attributes and nodes needed for the particular unit test, no more no less.
* if you need easy access to a particular node use a `ref` attribute on the node, since that is not used in our xml.
