#include "..\script_component.hpp"

params ["_func", "_data"];

switch (_func) do {
    case "load": {
        (parseSimpleArray _data) params ["_id", "_class", "_data"];
        [_id, _class, _data] call FUNC(objects_load);
    };
    case "done": {
        // objects_load applies each object's state a frame after creating it, and applying state clears
        // ACE cargo, so the cargo is loaded a frame later still, once every object's state is in place
        [FUNC(objects_loaded)] call CBA_fnc_execNextFrame;
    };
};
