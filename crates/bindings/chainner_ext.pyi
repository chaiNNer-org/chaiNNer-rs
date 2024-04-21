from __future__ import annotations

from enum import Enum
from typing import Dict, List, Literal

import numpy as np

# pylint: disable=unused-argument,missing-class-docstring,missing-function-docstring

def fill_alpha_fragment_blur(
    img: np.ndarray, threshold: float, iterations: int, fragment_count: int
) -> np.ndarray: ...
def fill_alpha_extend_color(
    img: np.ndarray, threshold: float, iterations: int
) -> np.ndarray: ...
def fill_alpha_nearest_color(
    img: np.ndarray, threshold: float, min_radius: int, anti_aliasing: bool
) -> np.ndarray: ...
def binary_threshold(
    img: np.ndarray, threshold: float, anti_aliasing: bool, extra_smoothness: float = 0.0
) -> np.ndarray: ...
def esdf(
    img: np.ndarray, radius: float, cutoff: float, pre_process: bool, post_process: bool
) -> np.ndarray: ...
def pixel_art_upscale(img: np.ndarray, algorithm: str, scale: int) -> np.ndarray: ...
def fast_gamma(img: np.ndarray, gamma: float) -> np.ndarray: ...

class UniformQuantization:
    @property
    def colors_per_channel(self) -> int: ...
    def __init__(self, colors_per_channel: int) -> None: ...

class PaletteQuantization:
    @property
    def channels(self) -> int: ...
    def colors(self) -> int: ...
    def __init__(self, palette: np.ndarray) -> None: ...

class DiffusionAlgorithm(Enum):
    FloydSteinberg = 0
    JarvisJudiceNinke = 1
    Stucki = 2
    Atkinson = 3
    Burkes = 4
    Sierra = 5
    TwoRowSierra = 6
    SierraLite = 7

def quantize(
    img: np.ndarray,
    quant: UniformQuantization | PaletteQuantization,
) -> np.ndarray: ...
def ordered_dither(
    img: np.ndarray,
    quant: UniformQuantization,
    map_size: int,
) -> np.ndarray: ...
def error_diffusion_dither(
    img: np.ndarray,
    quant: UniformQuantization | PaletteQuantization,
    algorithm: DiffusionAlgorithm,
) -> np.ndarray: ...
def riemersma_dither(
    img: np.ndarray,
    quant: UniformQuantization | PaletteQuantization,
    history_length: int,
    decay_ratio: float,
) -> np.ndarray: ...

class ResizeFilter(Enum):
    Nearest = 0
    Box = 8
    Linear = 1
    Hermite = 9
    CubicCatrom = 2
    CubicMitchell = 3
    CubicBSpline = 6
    Hamming = 10
    Hann = 11
    Lanczos = 4
    Lagrange = 7
    Gauss = 5

def resize(
    img: np.ndarray,
    new_size: tuple[int, int],
    filter: ResizeFilter,
    gamma_correction: bool,
) -> np.ndarray: ...

# Regex

class RustRegex:
    @property
    def pattern(self) -> str: ...
    @property
    def groups(self) -> int: ...
    @property
    def groupindex(self) -> Dict[str, int]: ...
    def __init__(self, pattern: str) -> None: ...
    def search(self, text: str, pos: int = 0) -> RegexMatch | None: ...
    def findall(self, text: str) -> List[RegexMatch]: ...
    def split(self, group_name: str) -> List[str]: ...
    def split_without_captures(self, group_name: str) -> List[str]: ...

class RegexMatch:
    @property
    def start(self) -> int: ...
    @property
    def end(self) -> int: ...
    @property
    def len(self) -> int: ...
    def get(self, group_index: int) -> MatchGroup | None: ...
    def get_by_name(self, group_name: str) -> MatchGroup | None: ...

class MatchGroup:
    @property
    def start(self) -> int: ...
    @property
    def end(self) -> int: ...
    @property
    def len(self) -> int: ...

# Clipboard

class Clipboard:
    def write_text(self, text: str) -> None: ...
    def write_image(
        self, image: np.ndarray, pixel_format: Literal["RGB", "BGR"]
    ) -> None: ...
    @staticmethod
    def create_instance() -> Clipboard: ...
