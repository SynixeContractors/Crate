#include "script_component.hpp"

if !(isMultiplayer) exitWith {};

addMissionEventHandler ["ExtensionCallback", {
    params ["_name", "_func", "_data"];
    if (_name != "crate:missions") exitWith {};
    switch (_func) do {
        case "set_date": {
            (parseSimpleArray _data) call FUNC(setDate);
        };
        case "intro_text": {
            [QGVAR(introText)] call CBA_fnc_globalEvent;
        };
    };
}];
