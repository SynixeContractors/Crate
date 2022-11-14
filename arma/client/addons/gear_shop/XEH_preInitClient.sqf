#include "script_component.hpp"

GVAR(boxes) = [];

[QGVAR(loader_register), FUNC(handleLoaderRegister)] call CBA_fnc_addEventHandler;
[QGVAR(loader_ready), FUNC(handleLoaderReady)] call CBA_fnc_addEventHandler;
[QGVAR(loader_error), FUNC(handleLoaderError)] call CBA_fnc_addEventHandler;
