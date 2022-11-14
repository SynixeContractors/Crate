#include "script_component.hpp"

if !(isMultiplayer) exitWith {};

[QGVAR(items_get), {
    EXTFUNC("gear:shop:items");
}] call CBA_fnc_addEventHandler;

addMissionEventHandler ["ExtensionCallback", {
    params ["_name", "_func", "_data"];
    if (_name != "crate:gear:shop") exitWith {};

    switch (_func) do {
        case "items:clear": {
            GVAR(items_importing) = createHashMap;
        };
        case "items:entry": {
            (parseSimpleArray _data) params ["_class", "_entry"];
            GVAR(items_importing) set [_class, _entry];
        };
        case "items:done": {
            GVAR(items) = +GVAR(items_importing);
            publicVariable QGVAR(items);
            INFO_1("Imported %1 items", count GVAR(items));
        };
    };
}];
