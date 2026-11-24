#include "..\script_component.hpp"

/*
 * Author: SynixeContractors
 * Counts everything in a body bag, as the shop classes it's stored in the locker as.
 *
 * Each class goes through shop_item_listing like a loadout does (see loadout_items), so an ACRE radio
 * is stored as its base radio rather than its unique instance (e.g. ACRE_PRC152_ID_3), which would
 * otherwise come back out of the locker as a second copy of a radio ACRE still tracks. Containers,
 * weapon attachments and loaded magazines are counted too, as clearing the bag deletes them all.
 *
 * Arguments:
 * 0: _bodybag <OBJECT>
 *
 * Return Value:
 * <HASHMAP> - Shop class to quantity
 *
 * Public: No
 */

params ["_bodybag"];

private _contents = createHashMap;

private _fnc_add = {
    params ["_class", ["_quantity", 1]];
    if (_class isEqualTo "") exitWith {};
    private _listing = [_class] call FUNC(shop_item_listing);
    _contents set [_listing, (_contents getOrDefault [_listing, 0]) + _quantity];
};

private _fnc_addCargo = {
    params ["_container"];
    {
        _x params ["_classes", "_quantities"];
        {
            [_x, _quantities select _forEachIndex] call _fnc_add;
        } forEach _classes;
    } forEach [
        getBackpackCargo _container,
        getItemCargo _container,
        getMagazineCargo _container
    ];
    // Weapons with their attachments and loaded magazines, which getWeaponCargo leaves out
    {
        _x params ["_weapon", "_muzzle", "_pointer", "_optic", ["_magazine", []], ["_magazine2", []], ["_bipod", ""]];
        {
            [_x] call _fnc_add;
        } forEach [_weapon, _muzzle, _pointer, _optic, _bipod];
        {
            if (_x isNotEqualTo []) then {
                [_x select 0] call _fnc_add;
            };
        } forEach [_magazine, _magazine2];
    } forEach weaponsItemsCargo _container;
};

[_bodybag] call _fnc_addCargo;
// Uniforms, vests and backpacks in the bag, whose contents the bag's own cargo doesn't list
{
    [_x select 1] call _fnc_addCargo;
} forEach everyContainer _bodybag;

_contents
