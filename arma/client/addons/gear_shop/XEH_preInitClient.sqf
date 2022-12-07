#include "script_component.hpp"

GVAR(boxes) = [];

[QGVAR(loader_register), FUNC(loader_register)] call CBA_fnc_addEventHandler;
[QGVAR(loader_ready), FUNC(loader_ready)] call CBA_fnc_addEventHandler;
[QGVAR(loader_error), FUNC(loader_error)] call CBA_fnc_addEventHandler;
