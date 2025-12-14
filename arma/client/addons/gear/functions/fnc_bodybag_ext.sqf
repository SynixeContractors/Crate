#include "script_component.hpp"

params ["_func", "_data"];

switch (_func) do {
    (parseSimpleArray _data) params ["_netId"];
    private _bodybag = objectFromNetId _netId;
    case "store:ok": {
        clearBackpackCargoGlobal _bodybag;
        clearItemCargoGlobal _bodybag;
        clearMagazineCargoGlobal _bodybag;
        clearWeaponCargoGlobal _bodybag;
        private _nearby = (getPosATL _bodybag) nearObjects ["CAManBase", 2];
        [QGVAR(notify), "Bodybag Stored", _nearby] call CBA_fnc_targetEvent;
    };
    case "store:err": {
        private _nearby = (getPosATL _bodybag) nearObjects ["CAManBase", 2];
        [QGVAR(notify), "Bodybag Storage Failed", _nearby] call CBA_fnc_targetEvent;
    };
};
