#include "script_component.hpp"

if !(isMultiplayer) exitWith {};

if !(GVAR(enabled)) exitWith {};
if !(GVAR(shop_enabled)) exitWith {};

EXTFUNC("gear:shop:items");
