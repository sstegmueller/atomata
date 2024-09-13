import math


def empirical_probability(state_count: int, total_count: int) -> float:
    """
    Calculate the empirical probability of a state.
    """
    assert total_count > 0
    return state_count / total_count


def shannon_entropy(probability_mass: list[float]) -> float:
    """
    Calculate the Shannon entropy of a probability mass function.
    The choice of base of the logarithm determines the unit of entropy.
    """
    return -sum(p * math.log2(p) for p in probability_mass if p > 0) / len(probability_mass)


def emergence(probaility_mass: list[float]) -> float:
    """
    Calculate the emergence of a probability mass function.
    Emergence = I_out / I_in, we assume I_in = 1 for random input.
    """
    return shannon_entropy(probaility_mass) / 1


def self_organization(probaility_mass: list[float]) -> float:
    """
    Calculate the self-organization of a probability mass function.
    Self-organization = I_in - I_out , we assume I_in = 1 for random input.
    """
    return 1 - shannon_entropy(probaility_mass)


def complexity(probaility_mass: list[float]) -> float:
    """
    Calculate the complexity of a probability mass function.
    Complexity = emergence * self-organization * a, where a is a constant to bound C to [0,1].
    """
    en = shannon_entropy(probaility_mass)
    emergence = en
    self_organization = 1 - en
    return emergence * self_organization * 4
