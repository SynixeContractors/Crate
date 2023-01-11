#include "script_component.hpp"

addMissionEventHandler ["ExtensionCallback", {
    params ["_name", "_component", "_data"];
    private _name = toLower _name;
    if (_name == "crate:restart") exitWith {
        EXTFUNC("restart");
    };
    if (_name != "crate:log") exitWith {};
    (parseSimpleArray _data) params ["_level", "_message"];
    diag_log text format ["[CRATE] (%1) %2: %3", _component, _level, _message];
}];
