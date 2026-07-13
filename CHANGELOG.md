# Changelog

## 1.0.0

### New

*  added `any()` and `all()` functions to the standard library. Both accept an object or collection
   and return true if any or all of the closures return true, respectively.

  ```ripsaw
  all([1, 2, 3]) -> |_i, num| { num > 2 }
  # returns false

  any([1, 2, 3]) -> |_i, num| { num > 2 }
  # returns true
  ```

## pre-1.0.0

Forked!
