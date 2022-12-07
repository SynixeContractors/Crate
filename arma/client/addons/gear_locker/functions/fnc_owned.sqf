#include "script_component.hpp"

params ["_item"];

GVAR(items) getOrDefault [_item, 0]
