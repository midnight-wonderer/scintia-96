#!/usr/bin/env python3
import os
import hashlib
import sys

def generate_random():
    data = os.urandom(16)
    words = [int.from_bytes(data[i:i+4], 'big') for i in range(0, 16, 4)]
    print("\n// Randomly generated key")
    print("const KEY: [u32; 4] = [%s];" % ', '.join('0x%08x' % w for w in words))

def generate_derived():
    try:
        material = input("Enter key material (string): ").strip()
    except EOFError:
        print("\nError: No input provided.")
        sys.exit(1)
        
    h = hashlib.sha512(('scintia-96:key:' + material).encode()).digest()
    words = [int.from_bytes(h[i:i+4], 'big') for i in range(0, 16, 4)]
    print(f"\n// derive_key(\"{material}\")")
    print("const KEY: [u32; 4] = [%s];" % ', '.join('0x%08x' % w for w in words))

def main():
    print("Scintia-96 Key Generator")
    print("-" * 25)
    print("1. Random (OS-level entropy)")
    print("2. Derived (Deterministic from string)")
    
    try:
        choice = input("\nSelect option [1/2]: ").strip()
    except EOFError:
        print("\nExiting.")
        sys.exit(0)
    
    if choice == '1':
        generate_random()
    elif choice == '2':
        generate_derived()
    else:
        print("Invalid choice.")

if __name__ == "__main__":
    main()
