#include "script_component.hpp"

params ["_name"];

if !(GVAR(loading)) exitWith {
    WARNING_1("attempting to ready %1 while not loading", _name);
};

GVAR(loaders) set [_name, true];
INFO_1("Loader %1 ready", _name);

scopeName "all_true";
private _all_true = true;
{
    if !(_y) exitWith {
        _all_true = false;
        breakTo "all_true";
    };
} forEach GVAR(loaders);

if (_all_true) then {
    GVAR(blurHandle) ppEffectAdjust [0];
    GVAR(blurHandle) ppEffectCommit 0.25;
    [{
        GVAR(blurHandle) ppEffectEnable false;
        ppEffectDestroy GVAR(blurHandle);
    }, [], 1] call CBA_fnc_waitAndExecute;

    GVAR(loading) = false;

    private _items = [keys EGVAR(shop,items)] call EFUNC(shop,items_allowed);
    _items append (keys EGVAR(locker,items));
    _items = _items - ["ItemRadioAcreFlagged"];

    [GVAR(loadingShop), _items, false] call ace_arsenal_fnc_initBox;
    [GVAR(loadingShop), player] call ace_arsenal_fnc_openBox;
    [GVAR(loadingShop), false] call ace_arsenal_fnc_removeBox;

    player enableSimulation true;
};
