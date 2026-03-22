#ifndef CRYPTO_ENGINE_H
#define CRYPTO_ENGINE_H

#include <stddef.h>
#include <stdbool.h>
#include <stdint.h>
#include <time.h>

// Maximum sizes for various crypto operations
#define CRYPTO_MAX_KEY_SIZE 64
#define CRYPTO_MAX_IV_SIZE 16
#define CRYPTO_MAX_HASH_SIZE 64
#define CRYPTO_MAX_SALT_SIZE 32
#define CRYPTO_MAX_TAG_SIZE 16

// Supported hash algorithms
typedef enum {
    CRYPTO_HASH_SHA256,
    CRYPTO_HASH_SHA512,
    CRYPTO_HASH_BLAKE2B,
    CRYPTO_HASH_MD5,        // For compatibility (not recommended)
    CRYPTO_HASH_SHA1        // For compatibility (not recommended)
} crypto_hash_algorithm_t;

// Supported encryption algorithms
typedef enum {
    CRYPTO_CIPHER_AES256_GCM,
    CRYPTO_CIPHER_AES256_CBC,
    CRYPTO_CIPHER_CHACHA20_POLY1305,
    CRYPTO_CIPHER_AES128_GCM,
    CRYPTO_CIPHER_AES128_CBC,
    CRYPTO_CIPHER_XOR          // Simple XOR (for testing/demo)
} crypto_cipher_algorithm_t;

// Key derivation functions
typedef enum {
    CRYPTO_KDF_PBKDF2,
    CRYPTO_KDF_SCRYPT,
    CRYPTO_KDF_ARGON2
} crypto_kdf_algorithm_t;

// Random number generator types
typedef enum {
    CRYPTO_RNG_SYSTEM,      // System RNG (/dev/urandom)
    CRYPTO_RNG_MERSENNE,    // Mersenne Twister (for deterministic testing)
    CRYPTO_RNG_CHACHA20     // ChaCha20-based CSPRNG
} crypto_rng_type_t;

// Hash context structure
typedef struct {
    crypto_hash_algorithm_t algorithm;
    void* internal_state;
    size_t total_length;
} crypto_hash_context_t;

// Cipher context structure
typedef struct {
    crypto_cipher_algorithm_t algorithm;
    uint8_t key[CRYPTO_MAX_KEY_SIZE];
    uint8_t iv[CRYPTO_MAX_IV_SIZE];
    size_t key_size;
    size_t iv_size;
    bool is_encrypt;
    void* internal_state;
} crypto_cipher_context_t;

// Key derivation parameters
typedef struct {
    crypto_kdf_algorithm_t algorithm;
    uint8_t salt[CRYPTO_MAX_SALT_SIZE];
    size_t salt_size;
    int iterations;
    size_t memory_cost;  // For scrypt/argon2
    int parallelism;     // For argon2
} crypto_kdf_params_t;

// Encrypted data structure
typedef struct {
    crypto_cipher_algorithm_t algorithm;
    uint8_t* ciphertext;
    size_t ciphertext_size;
    uint8_t iv[CRYPTO_MAX_IV_SIZE];
    size_t iv_size;
    uint8_t tag[CRYPTO_MAX_TAG_SIZE];  // For authenticated encryption
    size_t tag_size;
    char* metadata;  // JSON-encoded metadata
} crypto_encrypted_data_t;

// Secure note structure for safe cloud storage
typedef struct {
    char* note_id;
    char* title_hash;      // Hash of title for identification
    crypto_encrypted_data_t* encrypted_content;
    crypto_encrypted_data_t* encrypted_metadata;
    time_t created_timestamp;
    time_t modified_timestamp;
    char* content_hash;    // Hash for integrity verification
    int version;
} crypto_secure_note_t;

// Crypto engine configuration
typedef struct {
    crypto_hash_algorithm_t default_hash;
    crypto_cipher_algorithm_t default_cipher;
    crypto_kdf_algorithm_t default_kdf;
    crypto_rng_type_t rng_type;
    
    // Security parameters
    int default_kdf_iterations;
    size_t default_key_size;
    bool always_verify_integrity;
    
    // Performance settings
    bool use_hardware_acceleration;
    int thread_count;
} crypto_config_t;

// Crypto engine state
typedef struct {
    crypto_config_t* config;
    void* rng_state;
    
    // Statistics
    int operations_performed;
    size_t bytes_processed;
    double total_time;
} crypto_engine_t;

// API Functions

// Engine lifecycle
crypto_engine_t* crypto_engine_create(crypto_config_t* config);
void crypto_engine_destroy(crypto_engine_t* engine);
int crypto_engine_init(crypto_engine_t* engine);

// Configuration
crypto_config_t* crypto_engine_create_config(void);
void crypto_engine_destroy_config(crypto_config_t* config);

// Random number generation
int crypto_engine_random_bytes(crypto_engine_t* engine, uint8_t* buffer, size_t size);
int crypto_engine_random_int(crypto_engine_t* engine, int min, int max);
char* crypto_engine_random_string(crypto_engine_t* engine, size_t length, const char* charset);

// Hashing
crypto_hash_context_t* crypto_engine_hash_init(crypto_engine_t* engine, crypto_hash_algorithm_t algorithm);
int crypto_engine_hash_update(crypto_hash_context_t* ctx, const uint8_t* data, size_t size);
int crypto_engine_hash_final(crypto_hash_context_t* ctx, uint8_t* hash, size_t* hash_size);
void crypto_engine_hash_destroy(crypto_hash_context_t* ctx);

// Convenience hash functions
char* crypto_engine_hash_string(crypto_engine_t* engine, const char* input, crypto_hash_algorithm_t algorithm);
char* crypto_engine_hash_file(crypto_engine_t* engine, const char* filepath, crypto_hash_algorithm_t algorithm);
bool crypto_engine_verify_hash(crypto_engine_t* engine, const char* input, const char* expected_hash, 
                               crypto_hash_algorithm_t algorithm);

// Key derivation
int crypto_engine_derive_key(crypto_engine_t* engine, const char* password, 
                             const crypto_kdf_params_t* params, uint8_t* key, size_t key_size);
crypto_kdf_params_t* crypto_engine_create_kdf_params(crypto_kdf_algorithm_t algorithm, int iterations);

// Symmetric encryption
crypto_cipher_context_t* crypto_engine_cipher_init(crypto_engine_t* engine, 
                                                   crypto_cipher_algorithm_t algorithm,
                                                   const uint8_t* key, size_t key_size,
                                                   const uint8_t* iv, size_t iv_size,
                                                   bool encrypt);

int crypto_engine_cipher_update(crypto_cipher_context_t* ctx, const uint8_t* input, size_t input_size,
                                uint8_t* output, size_t* output_size);
int crypto_engine_cipher_final(crypto_cipher_context_t* ctx, uint8_t* output, size_t* output_size,
                               uint8_t* tag, size_t* tag_size);
void crypto_engine_cipher_destroy(crypto_cipher_context_t* ctx);

// High-level encryption/decryption
crypto_encrypted_data_t* crypto_engine_encrypt_data(crypto_engine_t* engine, 
                                                    const uint8_t* plaintext, size_t plaintext_size,
                                                    const char* password);
uint8_t* crypto_engine_decrypt_data(crypto_engine_t* engine, 
                                    const crypto_encrypted_data_t* encrypted,
                                    const char* password, size_t* plaintext_size);

// File encryption/decryption
int crypto_engine_encrypt_file(crypto_engine_t* engine, const char* input_file, 
                              const char* output_file, const char* password);
int crypto_engine_decrypt_file(crypto_engine_t* engine, const char* input_file, 
                              const char* output_file, const char* password);

// Secure note management (for cloud storage)
crypto_secure_note_t* crypto_engine_create_secure_note(crypto_engine_t* engine, 
                                                       const char* title, const char* content,
                                                       const char* password);
char* crypto_engine_decrypt_secure_note(crypto_engine_t* engine, 
                                        const crypto_secure_note_t* note, 
                                        const char* password);
int crypto_engine_update_secure_note(crypto_engine_t* engine, crypto_secure_note_t* note,
                                     const char* new_content, const char* password);

// Secure note serialization (for cloud storage)
char* crypto_engine_serialize_secure_note(const crypto_secure_note_t* note);
crypto_secure_note_t* crypto_engine_deserialize_secure_note(const char* serialized_data);

// Memory security
void crypto_engine_secure_zero(void* ptr, size_t size);
char* crypto_engine_secure_strdup(const char* str);
void crypto_engine_secure_free(void* ptr, size_t size);

// Integrity verification
bool crypto_engine_verify_data_integrity(crypto_engine_t* engine, 
                                         const uint8_t* data, size_t size,
                                         const char* expected_hash);
char* crypto_engine_create_data_signature(crypto_engine_t* engine, 
                                          const uint8_t* data, size_t size,
                                          const char* secret_key);

// Password utilities
bool crypto_engine_is_password_strong(const char* password);
int crypto_engine_password_strength_score(const char* password);
char* crypto_engine_generate_password(crypto_engine_t* engine, int length, bool include_symbols);

// Base64 encoding/decoding (for safe text representation)
char* crypto_engine_base64_encode(const uint8_t* input, size_t input_size);
uint8_t* crypto_engine_base64_decode(const char* input, size_t* output_size);

// Hexadecimal encoding/decoding
char* crypto_engine_hex_encode(const uint8_t* input, size_t input_size);
uint8_t* crypto_engine_hex_decode(const char* input, size_t* output_size);

// Benchmarking and performance
void crypto_engine_benchmark_hashing(crypto_engine_t* engine, size_t data_size, int iterations);
void crypto_engine_benchmark_encryption(crypto_engine_t* engine, size_t data_size, int iterations);
void crypto_engine_get_performance_stats(crypto_engine_t* engine, double* ops_per_second, 
                                         double* bytes_per_second);

// Error handling
typedef enum {
    CRYPTO_ERROR_NONE = 0,
    CRYPTO_ERROR_INVALID_ALGORITHM,
    CRYPTO_ERROR_INVALID_KEY_SIZE,
    CRYPTO_ERROR_INVALID_INPUT,
    CRYPTO_ERROR_MEMORY_ERROR,
    CRYPTO_ERROR_ENCRYPTION_FAILED,
    CRYPTO_ERROR_DECRYPTION_FAILED,
    CRYPTO_ERROR_HASH_FAILED,
    CRYPTO_ERROR_KDF_FAILED,
    CRYPTO_ERROR_VERIFICATION_FAILED,
    CRYPTO_ERROR_RNG_FAILED
} crypto_error_t;

const char* crypto_engine_error_string(crypto_error_t error);
crypto_error_t crypto_engine_get_last_error(crypto_engine_t* engine);

// Utility functions
size_t crypto_engine_get_hash_size(crypto_hash_algorithm_t algorithm);
size_t crypto_engine_get_key_size(crypto_cipher_algorithm_t algorithm);
size_t crypto_engine_get_iv_size(crypto_cipher_algorithm_t algorithm);
const char* crypto_engine_algorithm_name(crypto_cipher_algorithm_t algorithm);
bool crypto_engine_is_authenticated_encryption(crypto_cipher_algorithm_t algorithm);

// Memory management for crypto structures
void crypto_engine_free_encrypted_data(crypto_encrypted_data_t* data);
void crypto_engine_free_secure_note(crypto_secure_note_t* note);
void crypto_engine_free_kdf_params(crypto_kdf_params_t* params);

#endif // CRYPTO_ENGINE_H