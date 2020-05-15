// Takes a .sch file as an input
// Parses it to make sure it has a global variable defining the part # for the assembly
// Asks if the BOM should be added if it doesn't exist
// If it already exists error/ask to update. Then runs the update routine instead
// Then parses each part
// Asks to make it if it doesn't exist
// Associates with the top level