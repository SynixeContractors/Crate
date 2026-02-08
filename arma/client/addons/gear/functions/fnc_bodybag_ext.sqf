#include "script_component.hpp"

params ["_func", "_data"];

(parseSimpleArray _data) params ["_netId"];
private _bodybag = objectFromNetId _netId;
private _nearby = (getPosATL _bodybag) nearObjects ["CAManBase", 2];

switch (_func) do {
    case "store:ok": {
        clearBackpackCargoGlobal _bodybag;
        clearItemCargoGlobal _bodybag;
        clearMagazineCargoGlobal _bodybag;
        clearWeaponCargoGlobal _bodybag;
        [QGVAR(notify), "Bodybag Stored", _nearby] call CBA_fnc_targetEvent;
    };
    case "store:err": {
        [QGVAR(notify), "Bodybag Storage Failed", _nearby] call CBA_fnc_targetEvent;
    };
};
