#include "script_component.hpp"

params ["_func", "_args"];

switch (_func) do {
    case "load": {
        (parseSimpleArray _data) params ["_id", "_class", "_group", "_data"];
        [_id, _class, _group, _data] call FUNC(units_load);
    };
    case "done": {
        {
            if ((_x getVariable [QGVAR(id), ""]) == "") then { continue };
            _x enableSimulationGlobal true;
        } forEach (allUnits - allPlayers);
        GVAR(units_ready) = true;
        EXTFUNC("campaigns:markers:load");
    };
};
