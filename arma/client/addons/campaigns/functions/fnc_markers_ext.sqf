#include "script_component.hpp"

params ["_func", "_data"];

switch (_func) do {
    case "load": {
        (parseSimpleArray _data) params ["_name", "_data"];
        [_name, _data] call FUNC(markers_load);
    };
    case "done": {
        GVAR(markers_ready) = true;
    };
};
