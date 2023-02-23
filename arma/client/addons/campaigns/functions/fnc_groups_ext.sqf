#include "script_component.hpp"

params ["_func", "_args"];

switch (_func) do {
    case "load": {
        (parseSimpleArray _data) params ["_id", "_data"];
        [_id, _data] call FUNC(groups_load);
    };
    case "done": {
        GVAR(groups_ready) = true;
        EXTFUNC("campaigns:units:load");
    };
};
