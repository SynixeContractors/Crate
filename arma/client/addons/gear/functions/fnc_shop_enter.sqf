#include "script_component.hpp"

params ["_locker", "_balance"];

GVAR(shop_locker) = createHashMapFromArray _locker;
GVAR(shop_balance) = _balance;

GVAR(shop_blurHandle) ppEffectAdjust [0];
GVAR(shop_blurHandle) ppEffectCommit 0.25;
[{
    GVAR(shop_blurHandle) ppEffectEnable false;
    ppEffectDestroy GVAR(shop_blurHandle);
}, [], 1] call CBA_fnc_waitAndExecute;

private _items = [keys GVAR(shop_items)] call FUNC(shop_items_allowed);
_items append (keys GVAR(shop_locker));
_items = _items - ["ItemRadioAcreFlagged"];

[GVAR(shop_box), _items, false] call ace_arsenal_fnc_initBox;
[GVAR(shop_box), player] call ace_arsenal_fnc_openBox;
[GVAR(shop_box), false] call ace_arsenal_fnc_removeBox;

player enableSimulation true;
