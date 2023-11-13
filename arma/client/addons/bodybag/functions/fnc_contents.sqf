#include "..\script_component.hpp"

params ["_bodybag"];

private _items = [];

_items pushBack (getBackpackCargo _bodybag);
_items pushBack (getItemCargo _bodybag);
_items pushBack (getMagazineCargo _bodybag);
_items pushBack (getWeaponCargo _bodybag);

private _contents = createHashMap;

{
    _x params ["_class", "_quantity"];
    private _existing = _contents get [_class, 0];
    _contents set [_class, _existing + _quantity];
} forEach _items;

_contents
