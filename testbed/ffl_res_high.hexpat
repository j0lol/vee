struct ResourcePartsInfo {
    u32 dataPos;
    u32 dataSize;
    u32 compressedSize;
    u8 compressLevel;
    u8 windowBits;
    u8 memoryLevel;
    u8 strategy;
};

struct ResourceShapeHeader {
    u32 maxSize[12];
    ResourcePartsInfo beard[4];
    ResourcePartsInfo hatNormal[132];
    ResourcePartsInfo hatCap[132];
    ResourcePartsInfo faceline[12];
    ResourcePartsInfo glass[1];
    ResourcePartsInfo mask[12];
    ResourcePartsInfo noseline[18];
    ResourcePartsInfo nose[18];
    ResourcePartsInfo hairNormal[132];
    ResourcePartsInfo hairCap[132];
    ResourcePartsInfo foreheadNormal[132];
    ResourcePartsInfo foreheadCap[132];
};

struct ResourceTextureHeader {
    u32 maxSize[11];
    ResourcePartsInfo beard[3];
    ResourcePartsInfo hat[132];
    ResourcePartsInfo eye[62];     // [80] in AFL
    ResourcePartsInfo eyebrow[24]; // [28] in AFL
    ResourcePartsInfo faceline[12];
    ResourcePartsInfo faceMakeup[12];
    ResourcePartsInfo glass[9]; // [20] in AFL
    ResourcePartsInfo mole[2];
    ResourcePartsInfo mouth[37]; // [52] in AFL
    ResourcePartsInfo mustache[6];
    ResourcePartsInfo noseline[18];
};

struct ResourceHeader { 
    u32 signature; // "FFRA"
    u32 version;
    u32 uncompressedBufferSize;
    u32 expandedBufferSize;
    u32 expanded;
    ResourceTextureHeader textureHeader;
    ResourceShapeHeader shapeHeader;
    u32 unknown[12]; // Always 0x0?
};


ResourceHeader resourceheader_at_0x00 @ 0x00;

// struct Vec3 {
//     float x;
//     float y;
//     float z;
// };

// struct BoundingBox {
//     Vec3 min;
//     Vec3 max;
// };

// struct ResourceShapeDataHeader {
//     u32 elementPos[6];
//     u32 elementSize[6];
//     BoundingBox boundingBox;
//     float transform[144]; // resshapehairtransform.size / 4
// };