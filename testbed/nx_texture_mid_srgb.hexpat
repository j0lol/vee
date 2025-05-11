enum AttributeType : u8 {
    AttributeType_Position = 0, // AttributeFormat_16_16_16_16_Float / _p0
    AttributeType_Normal=1,   // AttributeFormat_10_10_10_2_Snorm / _n0
    AttributeType_Uv=2,       // AttributeFormat_16_16_Float / _u0
    AttributeType_Tangent=3,  // AttributeFormat_8_8_8_8_Snorm / _t0
    AttributeType_Param=4,    // AttributeFormat_8_8_8_8_Unorm / _c0 (Color)
    AttributeType_End=5
};

enum ResourceMemoryLevel : u8 {
    ResourceMemoryLevel_1=1,
    ResourceMemoryLevel_Min=1,
    ResourceMemoryLevel_2=2,
    ResourceMemoryLevel_3=3,
    ResourceMemoryLevel_4=4,
    ResourceMemoryLevel_5=5,
    ResourceMemoryLevel_6=6,
    ResourceMemoryLevel_7=7,
    ResourceMemoryLevel_8=8,
    ResourceMemoryLevel_Default=8,
    ResourceMemoryLevel_9=9,
    ResourceMemoryLevel_Max=9
};

enum ResourceCompressLevel : u8 {
    ResourceCompressLevel_0=0,   // Z_NO_COMPRESSION
    ResourceCompressLevel_Min=0,
    ResourceCompressLevel_1=1,   // Z_BEST_SPEED
    ResourceCompressLevel_2=2,
    ResourceCompressLevel_3=3,
    ResourceCompressLevel_4=4,
    ResourceCompressLevel_5=5,
    ResourceCompressLevel_6=6,
    ResourceCompressLevel_Default=6,
    ResourceCompressLevel_7=7,
    ResourceCompressLevel_8=8,
    ResourceCompressLevel_9=9,   // Z_BEST_COMPRESSION
    ResourceCompressLevel_Max=9
};

struct ResourceCommonAttribute { // Size = 0x10
    // Same size as FFLiResourcePartsInfo, identical up to compressLevel.
    u32 offset;
    u32 size;
    u32 compressedSize;
    u8 compressLevel; // ResourceCompressLevel
    // FFLiResourcePartsInfo: u8 windowBits; // FFLiResourceWindowBits
    u8 memoryLevel;   // ResourceMemoryLevel / Verified but unused.
    // FFLiResourcePartsInfo: u8 strategy;   // FFLiResourceStrategy
    u8 pad[2];        // Unused padding for alignment.
};






 enum ResourceTextureType : u8 {
    ResourceTextureType_Hat=0,
    ResourceTextureType_Eye=1,
    ResourceTextureType_Eyebrow=2,
    ResourceTextureType_Beard=3,
    ResourceTextureType_Wrinkle=4,
    ResourceTextureType_Make=5,
    ResourceTextureType_Glass=6,
    ResourceTextureType_Mole=7,
    ResourceTextureType_Mouth=8,
    ResourceTextureType_Mustache=9,
    ResourceTextureType_Noseline=10,
    ResourceTextureType_End=11
} ;

 struct ResourceTextureAttribute { // Size = 0xc
    // Same size as FFLiResourceTextureFooter, not identical.
    u32 alignment;
    u16 width;
    u16 height;
    u8 format;   // ResourceTextureFormat
    u8 mipCount;
    u8 tileMode; // ResourceTileMode
    u8 _pad[1];  // Unused padding for alignment.
} ;

 enum ResourceTileMode : u8 {
    ResourceTileMode_Optimal=0,
    ResourceTileMode_Linear=1,
    ResourceTileMode_End=2
} ;

enum ResourceTextureFormat : u8 {
    // Matches FFL.
    ResourceTextureFormat_R8_Unorm=0,
    ResourceTextureFormat_R8_B8_Unorm=1,
    ResourceTextureFormat_R8_G8_B8_A8_Unorm=2,
    ResourceTextureFormat_BC4_Unorm=3,
    ResourceTextureFormat_BC5_Unorm=4,
    ResourceTextureFormat_BC7_Unorm=5,
    ResourceTextureFormat_Astc4x4_Unorm=6,
    ResourceTextureFormat_End=7
};

struct TextureElement { // ResourceTextureHeader::Element / Size = 0x1c
     ResourceCommonAttribute common;
     ResourceTextureAttribute texture;
};

struct ResourceTextureHeader { // Size = 0x2428
    u32 signature;          // 0x5254464e / "NFTR"
    u32 version;            // 1
    u32 fileSize;           // Includes size of header.
    u32 maxSize[11];        // ResourceTextureType_End
    u32 maxAlignment[11];   // Either 8 or 16.
    TextureElement hat[132];     // HairType_End
    TextureElement eye[62];      // EyeType_ResourceCount
    TextureElement eyebrow[24];  // EyebrowType_End
    TextureElement beard[2];     // BeardType_TextureCount
    TextureElement wrinkle[12];  // FacelineWrinkle_End
    TextureElement make[12];     // FacelineMake_End
    TextureElement glass[20];    // GlassType_End
    TextureElement mole[2];      // MoleType_End
    TextureElement mouth[37];    // MouthType_ResourceCount
    TextureElement mustache[6];  // MustacheType_End
    TextureElement noseline[18]; // NoseType_End
};


ResourceTextureHeader resourcetextureheader_at_0x00 @ 0x00;