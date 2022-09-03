from hypothesis import strategies

from tests.exact_tests.strategies import points

points = points
points_lists = strategies.lists(points,
                                min_size=1)
two_or_more_points_lists = strategies.lists(points,
                                            min_size=2)
