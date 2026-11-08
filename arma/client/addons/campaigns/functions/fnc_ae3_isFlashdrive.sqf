#include "..\script_component.hpp"

/*
 * Author: SynixeContractors
 * Checks whether a class is an AE3 flash drive. Laptops also set ae3_item in their config, so checking that alone
 * would treat them as flash drives.
 *
 * Arguments:
 * 0: _class <STRING> - Object class name
 *
 * Return Value:
 * <BOOL> - True if the class is a flash drive
 *
 * Public: No
 */

params ["_class"];

getText (configFile >> "CfgVehicles" >> _class >> "ae3_item") == "Item_FlashDisk_AE3"
